use sensor_scenario::iggy::IggyConfig;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = IggyConfig::from_env()?;
    let _client = config.connect().await?;

    // TODO: poll Iggy and stream UavReading messages as SSE to the browser

    Ok(())
}
