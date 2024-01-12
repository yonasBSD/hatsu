use activitypub_federation::config::Data;
use axum::{debug_handler, http::StatusCode, response::IntoResponse, Json};
use hatsu_apub::actors::ApubUser;
use hatsu_db_schema::prelude::User;
use hatsu_utils::{AppData, AppError};
use sea_orm::*;
use std::ops::Deref;

use crate::entities::{CreateRemoveAccount, CreateRemoveAccountResult};

/// Create Account
#[utoipa::path(
    post,
    tag = "hatsu::admin",
    path = "/api/v0/admin/create-account",
    responses(
        (status = CREATED, description = "create succesfully", body = CreateRemoveAccountResult),
        (status = BAD_REQUEST, description = "error", body = AppError)
    ),
    security(("api_key" = ["token"]))
)]
#[debug_handler]
pub async fn create_account(
    data: Data<AppData>,
    Json(payload): Json<CreateRemoveAccount>,
) -> Result<impl IntoResponse, AppError> {
    match User::find_by_id(
        hatsu_utils::url::generate_user_url(data.domain(), &payload.name)?.to_string(),
    )
    .one(&data.conn)
    .await?
    {
        Some(account) => Err(AppError::new(
            format!("The account already exists: {}", account.name),
            None,
            Some(StatusCode::BAD_REQUEST),
        )),
        None => {
            let account = ApubUser::new(data.domain(), &payload.name).await?;
            let account = account
                .deref()
                .clone()
                .into_active_model()
                .insert(&data.conn)
                .await?;
            Ok((
                StatusCode::CREATED,
                Json(CreateRemoveAccountResult {
                    name: account.name.clone(),
                    message: format!("The account was successfully created: {}", account.name),
                }),
            ))
        }
    }
}
