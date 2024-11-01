use axum::extract::{Json, Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::Extension;
use axum_macros::debug_handler;
use sqlx;
use tokio::sync::oneshot;
use tracing::instrument;

use crate::core::error::Error;
use crate::core::state::AppState;
use crate::ns::dispatch::{Command, EditDispatch, IntermediateDispatch, NewDispatch};
use crate::types::response::{Dispatch, DispatchStatus};
use crate::utils::auth::User;

#[instrument(skip_all)]
pub(crate) async fn dispatch_options(
    State(state): State<AppState>,
) -> Result<(HeaderMap, StatusCode), Error> {
    let mut headers = HeaderMap::new();

    let nations = HeaderValue::from_str(&state.client.get_nation_names().await.join(","))?;

    headers.insert("X-Nations", nations);
    headers.insert("Allow", HeaderValue::from_static("OPTIONS, POST"));

    Ok((headers, StatusCode::NO_CONTENT))
}

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

// #[debug_handler]
#[instrument(skip(state, user))]
pub(crate) async fn post_dispatch(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(params): Json<NewDispatch>,
) -> Result<(StatusCode, Json<DispatchStatus>), Error> {
    if !user.claims.contains(&"dispatches.create".to_string()) {
        return Err(Error::Unauthorized);
    }

    let job = state
        .queue_dispatch("add", sqlx::types::Json(params.clone()))
        .await?;

    let dispatch = IntermediateDispatch::add(job.id, user.username, params)?;

    let (tx, rx) = oneshot::channel();

    state
        .dispatch_sender
        .send(Command::new(dispatch, tx))
        .await
        .unwrap();

    match rx.await {
        Ok(_) => Ok((StatusCode::ACCEPTED, Json(job))),
        Err(_e) => Err(Error::Internal),
    }
}

// #[debug_handler]
#[instrument(skip(state, user))]
pub(crate) async fn edit_dispatch(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<i32>,
    Json(params): Json<EditDispatch>,
) -> Result<(StatusCode, Json<DispatchStatus>), Error> {
    if !user.claims.contains(&"dispatches.edit".to_string()) {
        return Err(Error::Unauthorized);
    }

    let nation = state.get_dispatch_nation(id).await?;

    let job = state
        .queue_dispatch("edit", sqlx::types::Json(params.clone()))
        .await?;

    let dispatch = IntermediateDispatch::edit(job.id, user.username, id, nation, params)?;

    let (tx, rx) = oneshot::channel();

    state
        .dispatch_sender
        .send(Command::new(dispatch, tx))
        .await
        .unwrap();

    match rx.await {
        Ok(_) => Ok((StatusCode::ACCEPTED, Json(job))),
        Err(_e) => Err(Error::Internal),
    }
}

// #[debug_handler]
#[instrument(skip(state, user))]
pub(crate) async fn remove_dispatch(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, Json<DispatchStatus>), Error> {
    if !user.claims.contains(&"dispatches.delete".to_string()) {
        return Err(Error::Unauthorized);
    }

    let nation = state.get_dispatch_nation(id).await?;

    let job = state
        .queue_dispatch("remove", sqlx::types::Json(id))
        .await?;

    let dispatch = IntermediateDispatch::delete(job.id, user.username, id, nation);

    let (tx, rx) = oneshot::channel();

    state
        .dispatch_sender
        .send(Command::new(dispatch, tx))
        .await
        .unwrap();

    match rx.await {
        Ok(_) => Ok((StatusCode::ACCEPTED, Json(job))),
        Err(_e) => Err(Error::Internal),
    }
}

// #[debug_handler]
#[instrument(skip(state))]
pub(crate) async fn get_queued_dispatch(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<DispatchStatus>, Error> {
    let status = state.get_dispatch_status(id).await?;

    Ok(Json(status))
}
