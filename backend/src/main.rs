#![feature(duration_constructors)]

use axum::{Router, routing::get};

use axum::debug_handler;
use axum::extract::{Json, Path, Query};
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum_response_cache::CacheLayer;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};
use serde::Serialize;
use serde_json::to_string;
use std::convert::Infallible;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use yahoo_finance_api as yahoo;

//#[cfg(debug_assertions)]
//use axum_extra::handler::debug_handler;

#[derive(Serialize, Debug)] // Derive Serialize and Debug for easy printing
struct Candlestick {
    // Using milliseconds since epoch, common in trading APIs
    timestamp: u128,
    open: f32,
    high: f32,
    low: f32,
    close: f32,
    volume: f32,
}

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
        let price_change_dist = Normal::new(0.0, 0.5).unwrap();
        let volume_dist = Normal::new(1000.0, 500_f32).unwrap();
        let mut last_close = 150_f32;
        let candle_interval = Duration::from_secs(1);
        let steps_per_candle = 60;

        loop {
            let start_timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();

            let open = last_close + price_change_dist.sample(&mut rng);
            let mut current_price = open;
            let mut high = open;
            let mut low = open;
            let mut total_volume_this_candle = 0_f32;

            for _ in 0..steps_per_candle {
                let change = price_change_dist.sample(&mut rng);
                current_price += change;
                high = high.max(current_price);
                low = low.min(current_price);
                let step_volume = volume_dist.sample(&mut rng);
                total_volume_this_candle += step_volume.max(0.0);
            }

            let close = current_price;
            let volume = total_volume_this_candle;
            last_close = close;

            let candle = Candlestick {
                timestamp: start_timestamp,
                open,
                high,
                low,
                close,
                volume,
            };

            let data = match to_string(&candle) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{e}");
                    continue;
                }
            };
            let event = Event::default().data(data).event("candle update");

            if sender.send(Ok(event)).await.is_err() {
                // The receiver was dropped, meaning the client disconnected
                eprintln!(
                    "Client disconnected or channel closed for ticker: {}",
                    &ticker
                );
                break; // Exit the spawned task loop
            }
            tokio::time::sleep(candle_interval).await;
        }
        //let it = normal
        //    .sample_iter(rng)
        //    .map(|num| Ok(Event::default().data(format!("{num:.2}"))));

        ////it.try_for_each(|e| { async sender.send(e).await});

        //for e in it {
        //    if sender.send(e).await.is_err() {
        //        break;
        //    }
        //}
        ////loop {
        ////    let num = normal.sample(&mut rng);
        ////    let event = Event::default().data(format!("{num:.2}"));
        ////    if sender.send(Ok(event)).await.is_err() {
        ////        break;
        ////    }
        ////    // tokio::sleep if not throttling
        ////}
    });

    let stream = ReceiverStream::new(receiver); //.throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::default())
}
