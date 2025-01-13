use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::models::{Video, Subscription, Channel};

pub struct AppState {
    pub client: Client,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LookupType {
    CustomUrl,
    Vanity,
    Username,
    Handle,
    ChannelId,
}

#[derive(Debug, Deserialize)]
pub struct ChannelLookupRequest {
    pub r#type: LookupType,
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct ChannelLookupResponse {
    pub channel: Channel,
    pub redirect_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginatedRequest {
    pub id: String,
    pub page_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlaylistItemsResponse {
    pub items: Vec<Video>,
    pub page_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionsResponse {
    pub items: Vec<Subscription>,
    pub page_token: Option<String>,
}