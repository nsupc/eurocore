use crate::core::error::Error;
use crate::core::state::AppState;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use tracing::instrument;

// #[debug_handler]
#[instrument(skip(state))]
pub(crate) async fn dispatch(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, Error> {
    let status = state.get_dispatch_status(id).await?;

    Ok(Json(status))
}