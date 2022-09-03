use axum::{
    http::StatusCode,
    response::Redirect,
    routing::{get, get_service},
    Router,
};

use tower_cookies::CookieManagerLayer;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

async fn handle_root() -> Redirect {
    Redirect::to("/-/")
}

mod login;

#[tokio::main]
async fn main() {
    let frontend_service = get_service(
        ServeDir::new("../frontend/dist")
            .append_index_html_on_directories(true)
            .fallback(ServeFile::new("../frontend/dist/index.html")),
    )
    .handle_error(|error| async move {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", error),
        )
    });

    let app = Router::new()
        .nest("/api/", api_router())
        .nest("/-/", frontend_service)
        .route("/", get(handle_root))
        .layer(CookieManagerLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        );

    tracing_subscriber::fmt::init();

    login::setup().await;

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn api_router() -> Router {
    Router::new().nest("/login", login::router())
}
