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

mod config;
mod login;

#[tokio::main]
async fn main() {
    let config = config::load_configuration().expect("Unable to load configuration");

    println!("{:#?}", config);

    let index_html = {
        let mut p = config.resources.clone();
        p.push("index.html");
        p
    };

    let frontend_service = get_service(
        ServeDir::new(&config.resources)
            .append_index_html_on_directories(true)
            .fallback(ServeFile::new(&index_html)),
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

    login::setup(&config).await;

    // run it with hyper on localhost:3000
    axum::Server::bind(&format!("0.0.0.0:{}", config.port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn api_router() -> Router {
    Router::new().nest("/login", login::router())
}
