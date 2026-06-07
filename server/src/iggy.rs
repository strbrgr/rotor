use iggy::prelude::*;
use std::{env, error::Error};

pub struct IggyConfig {
    pub root_username: String,
    pub root_password: String,
    pub stream_name: String,
    pub topic_name: String,
    pub partition_id: u32,
}

impl IggyConfig {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            root_username: env::var("IGGY_ROOT_USERNAME")
                .unwrap_or_else(|_| DEFAULT_ROOT_USERNAME.to_string()),
            root_password: env::var("IGGY_ROOT_PASSWORD")
                .map_err(|_| "IGGY_ROOT_PASSWORD must be set (see .env)")?,
            stream_name: env::var("IGGY_STREAM_NAME")
                .map_err(|_| "IGGY_STREAM_NAME must be set (see .env)")?,
            topic_name: env::var("IGGY_TOPIC_NAME")
                .map_err(|_| "IGGY_TOPIC_NAME must be set (see .env)")?,
            partition_id: env::var("IGGY_PARTITION_ID")
                .map_err(|_| "IGGY_PARTITION_ID must be set (see .env)")?
                .parse::<u32>()
                .map_err(|_| "IGGY_PARTITION_ID must be a valid u32")?,
        })
    }

    pub async fn connect(&self) -> Result<IggyClient, Box<dyn Error>> {
        let client = IggyClient::default();
        client.connect().await?;
        client
            .login_user(&self.root_username, &self.root_password)
            .await?;
        Ok(client)
    }
}
