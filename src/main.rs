mod dangerous_open_internet;
mod ridiculous_routing;

use axum::{
    Router,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};

use crate::{
    dangerous_open_internet::manifest,
    ridiculous_routing::{dest, key, v6_dest, v6_key},
};

async fn hello_world() -> &'static str {
    "Hello, bird!"
}

async fn seek() -> impl IntoResponse {
    (
        StatusCode::FOUND,
        [(
            header::LOCATION,
            "https://www.youtube.com/watch?v=9Gc4QTqslN4",
        )],
        "",
    )
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/-1/seek", get(seek))
        .route("/2/dest", get(dest))
        .route("/2/key", get(key))
        .route("/2/v6/dest", get(v6_dest))
        .route("/2/v6/key", get(v6_key))
        .route("/5/manifest", post(manifest));

    Ok(router.into())
}
