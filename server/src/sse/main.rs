use axum::{
    Router,
    extract::State,
    response::{Sse, sse::Event},
    routing::get,
};
use iggy::prelude::*;
use sensor_scenario::{UavReading, iggy::IggyConfig};
use std::{convert::Infallible, error::Error, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio::time::sleep;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    tx: Arc<broadcast::Sender<UavReading>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = IggyConfig::from_env()?;
    let client = config.connect().await?;

    let (tx, _) = broadcast::channel(100);
    let tx = Arc::new(tx);

    tokio::spawn(consume_messages(client, config, Arc::clone(&tx)));

    let app = Router::new()
        .route("/events", get(sse_handler))
        .with_state(AppState { tx });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;
    info!("SSE server listening on 127.0.0.1:3001");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    Sse::new(uav_stream(rx)).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

fn uav_stream(
    rx: broadcast::Receiver<UavReading>,
) -> impl futures_util::Stream<Item = Result<Event, Infallible>> {
    BroadcastStream::new(rx)
        .filter_map(|r| r.ok())
        .map(|reading| {
            let data = serde_json::to_string(&reading).unwrap_or_default();
            Ok(Event::default().data(data))
        })
}

async fn consume_messages(
    client: IggyClient,
    config: IggyConfig,
    tx: Arc<broadcast::Sender<UavReading>>,
) {
    let interval = Duration::from_millis(500);
    info!(
        "Consuming from stream: {}, topic: {}, partition: {}",
        config.stream_name, config.topic_name, config.partition_id,
    );

    let stream_id = Identifier::try_from(config.stream_name.as_str()).unwrap();
    let topic_id = Identifier::try_from(config.topic_name.as_str()).unwrap();
    let mut offset = 0;
    let messages_per_batch = 10;
    let consumer = Consumer::default();

    loop {
        let result = client
            .poll_messages(
                &stream_id,
                &topic_id,
                Some(config.partition_id),
                &consumer,
                &PollingStrategy::offset(offset),
                messages_per_batch,
                false,
            )
            .await;

        let polled = match result {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to poll messages: {e}");
                sleep(interval).await;
                continue;
            }
        };

        if polled.messages.is_empty() {
            sleep(interval).await;
            continue;
        }

        offset += polled.messages.len() as u64;
        for message in polled.messages {
            match serde_json::from_slice::<UavReading>(&message.payload) {
                Ok(reading) => {
                    let _ = tx.send(reading);
                }
                Err(e) => error!("Failed to deserialize message: {e}"),
            }
        }
        sleep(interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use uuid::Uuid;

    fn sample_reading() -> UavReading {
        UavReading {
            id: Uuid::nil(),
            x: 1.0,
            y: 2.0,
            z: 3.0,
            created_at: 1000,
        }
    }

    #[tokio::test]
    async fn stream_emits_one_event_per_reading() {
        let (tx, rx) = broadcast::channel(10);
        tx.send(sample_reading()).unwrap();
        tx.send(sample_reading()).unwrap();
        drop(tx);

        let events: Vec<_> = uav_stream(rx).collect().await;
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn stream_closes_when_sender_drops() {
        let (tx, rx) = broadcast::channel(10);
        drop(tx);

        let events: Vec<_> = uav_stream(rx).collect().await;
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn lagged_receiver_drops_missed_messages_and_continues() {
        let (tx, rx) = broadcast::channel(2);
        for i in 0..5 {
            let mut r = sample_reading();
            r.x = i as f32;
            tx.send(r).unwrap();
        }
        drop(tx);

        let events: Vec<_> = uav_stream(rx).collect().await;
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn reading_serializes_to_valid_json() {
        let reading = sample_reading();
        let json = serde_json::to_string(&reading).unwrap();
        assert!(json.contains("\"x\":1.0"));
        assert!(json.contains("\"y\":2.0"));
        assert!(json.contains("\"z\":3.0"));
        assert!(json.contains("\"created_at\":1000"));
    }
}
