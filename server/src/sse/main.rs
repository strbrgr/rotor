use iggy::prelude::*;
use sensor_scenario::{UavReading, iggy::IggyConfig};
use std::error::Error;
use tracing::info;

use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = IggyConfig::from_env()?;
    let client = config.connect().await?;

    consume_messages(&client, &config).await;

    Ok(())
}

async fn consume_messages(client: &IggyClient, config: &IggyConfig) -> Result<(), Box<dyn Error>> {
    let interval = Duration::from_millis(500);
    info!(
        "Messages will be consumed from stream: {}, topic: {}, partition: {} with interval {} ms.",
        config.stream_name,
        config.topic_name,
        config.partition_id,
        interval.as_millis()
    );

    let stream_id = Identifier::try_from(config.stream_name.as_str())?;
    let topic_id = Identifier::try_from(config.topic_name.as_str())?;
    let mut offset = 0;
    let messages_per_batch = 10;
    let consumer = Consumer::default();

    loop {
        let polled_messages = client
            .poll_messages(
                &stream_id,
                &topic_id,
                Some(config.partition_id),
                &consumer,
                &PollingStrategy::offset(offset),
                messages_per_batch,
                false,
            )
            .await?;

        if polled_messages.messages.is_empty() {
            info!("No messages found.");
            sleep(interval).await;
            continue;
        }

        offset += polled_messages.messages.len() as u64;
        for message in polled_messages.messages {
            handle_message(&message)?;
        }
        sleep(interval).await;
    }
}

fn handle_message(message: &IggyMessage) -> Result<(), Box<dyn Error>> {
    let _ = serde_json::from_slice::<UavReading>(&message.payload)?;

    Ok(())
}
