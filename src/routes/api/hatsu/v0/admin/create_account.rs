use activitypub_federation::config::Data;
use anyhow::anyhow;
use axum::{
    debug_handler,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::*;
use serde::{Deserialize, Serialize};

use crate::{
    AppData,
    AppError,
    entities::{
        prelude::*,
        user::Model as DbUser,
    },
};

#[derive(Deserialize)]
pub struct CreateAccount {
    token: Option<String>,
    name: String,
}

#[derive(Serialize)]
pub struct CreateAccountResult {
    name: String,
    message: String,
}

#[debug_handler]
pub async fn create_account(
    data: Data<AppData>,
    Json(payload): Json<CreateAccount>,
) -> Result<impl IntoResponse, AppError> {
    match payload.token {
        token if (token.is_some() && token == data.env.hatsu_access_token) => {
            match User::find_by_id(format!("https://{}/u/{}", data.domain(), payload.name))
                .one(&data.conn)
                .await? {
                    Some(account) => Ok((StatusCode::BAD_REQUEST, Json(CreateAccountResult {
                        name: account.name.clone(),
                        message: format!("The account already exists: {}", account.name),
                    }))),
                    _ => {
                        let account = DbUser::new(data.domain(), &payload.name).await?;
                        let account = account.into_active_model().insert(&data.conn).await?;
                        Ok((StatusCode::CREATED, Json(CreateAccountResult {
                            name: account.name.clone(),
                            message: format!("The account was successfully created: {}", account.name),
                        })))
                    }
                }
        }
        // TODO: StatusCode::FORBIDDEN
        _ => Err(anyhow!("Access Token Authentication Failed").into())
    }
}
