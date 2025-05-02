use axum::{Router, routing::get};

use axum::debug_handler;
use axum::extract::{Json, Path, Query};
use axum::response::IntoResponse;
use axum_response_cache::CacheLayer;
use yahoo_finance_api as yahoo;

#[dotenvy::load]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/quote/{ticker}",
            get(fetch_quote).layer(CacheLayer::with_lifespan(60 * 5)),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn fetch_quote(Path(ticker): Path<String>) -> impl IntoResponse {
    let provider = yahoo::YahooConnector::new().unwrap();

    let x = provider.get_latest_quotes(&ticker, "1d").await;

    dbg!(&x);

    let x = x.unwrap().last_quote().unwrap().close;

    format!("{x:?}")
}
