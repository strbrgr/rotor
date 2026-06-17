use iggy::prelude::*;
use questdb::ingress::{Buffer, Sender, TimestampNanos};
use rotor_server::{UavReading, iggy::IggyConfig};
use std::env;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

struct Config {
    iggy: IggyConfig,
    qdb_client_conf: String,
}

impl Config {
    fn from_env() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            iggy: IggyConfig::from_env()?,
            qdb_client_conf: env::var("QDB_CLIENT_CONF")
                .map_err(|_| "QDB_CLIENT_CONF must be set (see .env)")?,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    let client = config.iggy.connect().await?;
    consume_messages(&client, &config).await
}

async fn consume_messages(client: &IggyClient, config: &Config) -> Result<(), Box<dyn Error>> {
    let interval = Duration::from_millis(500);
    info!(
        "Messages will be consumed from stream: {}, topic: {}, partition: {} with interval {} ms.",
        config.iggy.stream_name,
        config.iggy.topic_name,
        config.iggy.partition_id,
        interval.as_millis()
    );

    let stream_id = Identifier::try_from(config.iggy.stream_name.as_str())?;
    let topic_id = Identifier::try_from(config.iggy.topic_name.as_str())?;
    let mut offset = 0;
    let messages_per_batch = 10;
    let consumer = Consumer::default();
    let mut sender = Sender::from_conf(config.qdb_client_conf.as_str())?;

    loop {
        let polled_messages = client
            .poll_messages(
                &stream_id,
                &topic_id,
                Some(config.iggy.partition_id),
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
            handle_message(&message, &mut sender)?;
        }
        sleep(interval).await;
    }
}

fn handle_message(message: &IggyMessage, sender: &mut Sender) -> Result<(), Box<dyn Error>> {
    let reading = serde_json::from_slice::<UavReading>(&message.payload)?;
    let mut buffer = Buffer::new(questdb::ingress::ProtocolVersion::V3);

    buffer
        .table("uav_position")?
        .symbol("sensor_id", reading.id.to_string())?
        .column_f64("x", reading.x.into())?
        .column_f64("y", reading.y.into())?
        .column_f64("z", reading.z.into())?
        .at(TimestampNanos::new(reading.created_at))?;

    sender.flush(&mut buffer)?;

    Ok(())
}
