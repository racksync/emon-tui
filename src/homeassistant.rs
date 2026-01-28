use anyhow::{Context, Result};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EntityState {
    #[allow(dead_code)]
    pub entity_id: String,
    pub state: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub attributes: serde_json::Value,
}

#[derive(Debug)]
pub struct HomeAssistant {
    url: String,
    token: String,
    client: reqwest::Client,
}

impl HomeAssistant {
    pub fn new(url: String, token: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        Self { url, token, client }
    }

    pub async fn get_state(&self, entity_id: &str) -> Result<EntityState> {
        let url = format!(
            "{}/api/states/{}",
            self.url.trim_end_matches('/'),
            entity_id
        );

        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .send()
            .await
            .with_context(|| format!("Failed to fetch entity: {}", entity_id))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Home Assistant API returned status {} for entity {}",
                response.status(),
                entity_id
            );
        }

        let state: EntityState = response
            .json()
            .await
            .with_context(|| format!("Failed to parse response for entity {}", entity_id))?;

        Ok(state)
    }
}
