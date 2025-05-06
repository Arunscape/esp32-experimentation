use axum::{Router, routing::get};

use axum::debug_handler;
use axum::extract::{Json, Path, Query};
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum_response_cache::CacheLayer;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
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
        )
        .route("/streaming_quote/{ticker}", get(quote_stream));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();
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

#[debug_handler]
async fn quote_stream(
    Path(ticker): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (sender, receiver) = mpsc::channel(10);

    tokio::spawn(async move {
        let mut rng = StdRng::from_os_rng();
        let normal = Normal::new(100_f32, 99_f32).unwrap();
        let it = normal
            .sample_iter(rng)
            .map(|num| Ok(Event::default().data(format!("{num:.2}"))));

        //it.try_for_each(|e| { async sender.send(e).await});

        for e in it {
            if sender.send(e).await.is_err() {
                break;
            }
        }
        //loop {
        //    let num = normal.sample(&mut rng);
        //    let event = Event::default().data(format!("{num:.2}"));
        //    if sender.send(Ok(event)).await.is_err() {
        //        break;
        //    }
        //    // tokio::sleep if not throttling
        //}
    });

    let stream = ReceiverStream::new(receiver).throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::default())
}
