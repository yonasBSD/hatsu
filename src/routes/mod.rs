use axum::{http::Response, response::IntoResponse, routing::get, Router};

mod activities;
mod api;
mod nodeinfo;
mod objects;
mod users;
mod well_known;

mod openapi;

// ./hatsu --version
async fn root() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    let codename = "01_ballade";

    Response::new(format!("Hatsu v{} \"{}\"", version, codename))
}

pub fn handler() -> Router {
    Router::new()
        .merge(activities::handler())
        .merge(api::handler())
        .merge(nodeinfo::handler())
        .merge(objects::handler())
        .merge(users::handler())
        .merge(well_known::handler())
        .merge(openapi::handler())
        .route("/", get(root))
}
