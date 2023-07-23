use activitypub_federation::{
    config::Data,
    fetch::webfinger::{build_webfinger_response, extract_webfinger_name, Webfinger},
};
use axum::{
    debug_handler,
    extract::Query,
    Json
};
use sea_orm::*;
use serde::Deserialize;
use url::Url;

use crate::{
    entities::{prelude::*, *},
    error::Error,
    AppData
};

#[derive(Deserialize)]
pub struct WebfingerQuery {
    resource: String,
}

#[debug_handler]
pub async fn webfinger(
    Query(query): Query<WebfingerQuery>,
    data: Data<AppData>,
) -> Result<Json<Webfinger>, Error> {
    tracing::info!("{}", &query.resource);

    let name = extract_webfinger_name(&query.resource, &data)?;
    let id = format!("https://{}/{}", data.domain(), &name);

    let user: Option<user::Model> = User::find_by_id(&id)
        .one(&data.conn)
        .await?;

    Ok(Json(build_webfinger_response(
        query.resource,
        Url::parse(&user.unwrap().id).unwrap()
    )))
}
