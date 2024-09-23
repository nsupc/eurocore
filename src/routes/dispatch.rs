use crate::core::error::Error;
use crate::core::state::AppState;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::Extension;
use tokio::sync::oneshot;
use tracing::instrument;

use crate::ns::dispatch::{Command, EditDispatch, IntermediateDispatch, NewDispatch, Response};
use crate::types::response::{Dispatch, DispatchHeader};
use crate::utils::auth::User;

#[instrument(skip(state))]
pub(crate) async fn get_dispatch(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Dispatch>, Error> {
    let dispatch = state.get_dispatch(id).await?;

    Ok(Json(dispatch))
}

#[instrument(skip(state))]
pub(crate) async fn get_dispatches(
    State(state): State<AppState>,
) -> Result<Json<Vec<Dispatch>>, Error> {
    let dispatches = state.get_dispatches(None).await?;

    Ok(Json(dispatches))
}

#[instrument(skip(state))]
pub(crate) async fn get_dispatches_by_nation(
    State(state): State<AppState>,
    Path(nation): Path<String>,
) -> Result<Json<Vec<Dispatch>>, Error> {
    let dispatches = state.get_dispatches(Some(nation)).await?;

    Ok(Json(dispatches))
}

#[instrument(skip(state, user))]
pub(crate) async fn post_dispatch(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(params): Json<NewDispatch>,
) -> Result<StatusCode, Error> {
    if !user.claims.contains(&"dispatches.create".to_string()) {
        return Err(Error::Unauthorized);
    }

    let dispatch = IntermediateDispatch::add(params)?;

    let (tx, rx) = oneshot::channel();

    state
        .dispatch_sender
        .send(Command::new(dispatch, tx))
        .await
        .unwrap();

    let _response = match rx.await {
        Ok(Response::Success) => (),
        Ok(Response::Error(e)) => return Err(e),
        _ => todo!(),
    };

    Ok(StatusCode::ACCEPTED)
}

#[instrument(skip(state, user))]
pub(crate) async fn edit_dispatch(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<i32>,
    Json(params): Json<EditDispatch>,
) -> Result<StatusCode, Error> {
    if !user.claims.contains(&"dispatches.edit".to_string()) {
        return Err(Error::Unauthorized);
    }

    let nation = state.get_dispatch_nation(id).await?;

    let dispatch = IntermediateDispatch::edit(id, nation, params)?;

    let (tx, rx) = oneshot::channel();

    state
        .dispatch_sender
        .send(Command::new(dispatch, tx))
        .await
        .unwrap();

    let _response = match rx.await {
        Ok(Response::Success) => (),
        Ok(Response::Error(e)) => return Err(e),
        _ => todo!(),
    };

    Ok(StatusCode::ACCEPTED)
}

#[instrument(skip(state, user))]
pub(crate) async fn remove_dispatch(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<i32>,
) -> Result<StatusCode, Error> {
    if !user.claims.contains(&"dispatches.delete".to_string()) {
        return Err(Error::Unauthorized);
    }

    let nation = state.get_dispatch_nation(id).await?;

    let dispatch = IntermediateDispatch::delete(id, nation);

    let (tx, rx) = oneshot::channel();

    state
        .dispatch_sender
        .send(Command::new(dispatch, tx))
        .await
        .unwrap();

    let _response = match rx.await {
        Ok(Response::Success) => (),
        Ok(Response::Error(e)) => return Err(e),
        _ => todo!(),
    };

    Ok(StatusCode::ACCEPTED)
}
