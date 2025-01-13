use axum::{
    routing::{get, post},
    Router,
    Json,
    response::Html,
    extract::State,
};
use std::env;
use std::sync::Arc;
use crate::youtube::{channels::{get_channel, LookupType as YTLookupType}, playlist_items::get_playlist_items, subscriptions::get_subscriptions};
use crate::youtubei::{resolve_url::{resolve_url, ResolveUrlResult}, browse::enrich_channel_data};
use super::types::{AppState, ChannelLookupRequest, ChannelLookupResponse, LookupType, PaginatedRequest, PlaylistItemsResponse, SubscriptionsResponse};
use super::error::ApiError;
use crate::errors::YouTubeError;

const MAX_RESULTS: u32 = 50;

#[cfg(test)]
fn get_api_key() -> String {
    dotenvy::dotenv().ok();
    env::var("API_KEY").expect("API_KEY must be set")
}

#[cfg(not(test))]
fn get_api_key() -> String {
    env::var("API_KEY").expect("API_KEY must be set")
}

async fn check_channel_status(client: &reqwest::Client, channel_id: &str) -> Result<Json<ChannelLookupResponse>, ApiError> {
    let api_key = get_api_key();
    match get_subscriptions(client, channel_id, &api_key, None, 1).await {
        Err(YouTubeError::AccountTerminated) => {
            Err(ApiError::NotFound("This channel has been terminated".to_string()))
        }
        Err(YouTubeError::AccountClosed) => {
            Err(ApiError::NotFound("This channel has been deleted".to_string()))
        }
        _ => Err(ApiError::NotFound("Channel not found".to_string()))
    }
}

async fn channel_handler(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<ChannelLookupRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<ChannelLookupResponse>, ApiError> {
    let Json(payload) = payload.map_err(|e| ApiError::InvalidRequest(e.to_string()))?;

    let (channel, redirect_url) = match payload.r#type {
        LookupType::CustomUrl => {
            // First get channel from +URL
            let plus_url = format!("youtube.com/+{}", payload.id);
            let plus_resolve_result = match resolve_url(&state.client, plus_url).await {
                Ok(Some(result)) => result,
                Ok(None) => return Err(ApiError::NotFound("Custom URL not found".to_string())),
                Err(YouTubeError::NotFound) => return Err(ApiError::NotFound("Custom URL not found".to_string())),
                Err(e) => return Err(ApiError::YouTubeError(e)),
            };

            println!("check: {:?}", plus_resolve_result);

            let browse_id = match plus_resolve_result {
                ResolveUrlResult::BrowseEndpoint { browse_id } => browse_id,
                ResolveUrlResult::UrlEndpoint { .. } => {
                    return Err(ApiError::InvalidRequest("Invalid custom URL - unexpected URL endpoint".to_string()))
                }
            };

            let api_key = get_api_key();
            let mut channel = get_channel(
                &state.client,
                YTLookupType::ChannelID(browse_id),
                &api_key,
            ).await?;

            // Try to enrich but continue if it fails
            if let Err(e) = enrich_channel_data(&state.client, &mut channel).await {
                eprintln!("Failed to enrich channel data for {}: {:?}", channel.user_id, e);
            }

            // Then check non-plus URL for redirect
            let url = format!("youtube.com/{}", payload.id.to_uppercase());
            let resolve_result = resolve_url(&state.client, url)
                .await
                .map_err(|e| ApiError::YouTubeError(e))?;

            let redirect_url = match resolve_result {
                Some(ResolveUrlResult::UrlEndpoint { url }) => Some(url),
                _ => None,
            };

            (channel, redirect_url)
        }
        LookupType::Vanity => {
            // Get the main vanity URL channel first
            let url = format!("youtube.com/{}", payload.id.to_uppercase());
            let resolve_result = resolve_url(&state.client, url)
                .await
                .map_err(|e| ApiError::YouTubeError(e))?;

            let main_channel_id = match resolve_result {
                Some(ResolveUrlResult::BrowseEndpoint { browse_id }) => browse_id,
                _ => return Err(ApiError::NotFound("Invalid vanity URL".to_string())),
            };

            // Check +URL, but only error if it points to the same channel
            let plus_url = format!("youtube.com/+{}", payload.id);
            if let Ok(Some(ResolveUrlResult::BrowseEndpoint { browse_id })) = resolve_url(&state.client, plus_url).await {
                if browse_id == main_channel_id {
                    return Err(ApiError::NotFound("Not a vanity URL".to_string()));
                }
            }

            // Check /user/, but only error if it points to the same channel
            let user_url = format!("youtube.com/user/{}", payload.id);
            if let Ok(Some(ResolveUrlResult::BrowseEndpoint { browse_id })) = resolve_url(&state.client, user_url).await {
                if browse_id == main_channel_id {
                    return Err(ApiError::NotFound("Not a vanity URL".to_string()));
                }
            }

            // If we get here, it's a valid vanity URL - return the channel
            let api_key = get_api_key();
            let mut channel = get_channel(
                &state.client,
                YTLookupType::ChannelID(main_channel_id),
                &api_key,
            ).await?;

            // Try to enrich but continue if it fails
            if let Err(e) = enrich_channel_data(&state.client, &mut channel).await {
                eprintln!("Failed to enrich channel data for {}: {:?}", channel.user_id, e);
            }

            (channel, None)
        }
        LookupType::Username => {
            let api_key = get_api_key();
            let mut channel = get_channel(
                &state.client,
                YTLookupType::Username(payload.id.clone()),
                &api_key,
            ).await?;

            // Try to enrich but continue if it fails
            if let Err(e) = enrich_channel_data(&state.client, &mut channel).await {
                eprintln!("Failed to enrich channel data for {}: {:?}", channel.user_id, e);
            }

            let mut redirect_url = None;
            if let Some(handle) = channel.handle.clone() {
                let url = format!("youtube.com/@{}", handle);
                let resolve_result = resolve_url(&state.client, url)
                    .await
                    .map_err(|e| ApiError::YouTubeError(e))?;

                redirect_url = match resolve_result {
                    Some(ResolveUrlResult::UrlEndpoint { url }) => Some(url),
                    _ => None,
                };
            }

            (channel, redirect_url)
        }
        LookupType::Handle => {
            let api_key = get_api_key();
            let mut channel = get_channel(
                &state.client,
                YTLookupType::Handle(payload.id.clone()),
                &api_key,
            ).await?;

            // Try to enrich but continue if it fails
            if let Err(e) = enrich_channel_data(&state.client, &mut channel).await {
                eprintln!("Failed to enrich channel data for {}: {:?}", channel.user_id, e);
            }

            let mut redirect_url = None;
            if let Some(handle) = channel.handle.clone() {
                let url = format!("youtube.com/@{}", handle);
                let resolve_result = resolve_url(&state.client, url)
                    .await
                    .map_err(|e| ApiError::YouTubeError(e))?;

                redirect_url = match resolve_result {
                    Some(ResolveUrlResult::UrlEndpoint { url }) => Some(url),
                    _ => None,
                };
            }

            (channel, redirect_url)
        }
        LookupType::ChannelId => {
            let api_key = get_api_key();
            let channel_result = get_channel(
                &state.client,
                YTLookupType::ChannelID(payload.id.clone()),
                &api_key,
            ).await;

            let mut channel = match channel_result {
                Ok(channel) => channel,
                Err(YouTubeError::NotFound) => {
                    return check_channel_status(&state.client, &payload.id).await;
                }
                Err(e) => return Err(ApiError::YouTubeError(e)),
            };

            if let Err(e) = enrich_channel_data(&state.client, &mut channel).await {
                eprintln!("Failed to enrich channel data for {}: {:?}", channel.user_id, e);
            }

            let mut redirect_url = None;
            if let Some(handle) = channel.handle.clone() {
                let url = format!("youtube.com/@{}", handle);
                let resolve_result = resolve_url(&state.client, url)
                    .await
                    .map_err(|e| ApiError::YouTubeError(e))?;

                redirect_url = match resolve_result {
                    Some(ResolveUrlResult::UrlEndpoint { url }) => Some(url),
                    _ => None,
                };
            }

            (channel, redirect_url)
        }
    };

    Ok(Json(ChannelLookupResponse {
        channel,
        redirect_url,
    }))
}

async fn playlist_items_handler(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<PaginatedRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<PlaylistItemsResponse>, ApiError> {
    let Json(payload) = payload.map_err(|e| ApiError::InvalidRequest(e.to_string()))?;

    let api_key = get_api_key();
    let (items, page_token) = get_playlist_items(
        &state.client,
        &payload.id,
        &api_key,
        payload.page_token.as_deref(),
        MAX_RESULTS,
    ).await?;

    Ok(Json(PlaylistItemsResponse {
        items,
        page_token,
    }))
}

async fn subscriptions_handler(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<PaginatedRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<SubscriptionsResponse>, ApiError> {
    let Json(payload) = payload.map_err(|e| ApiError::InvalidRequest(e.to_string()))?;

    let api_key = get_api_key();
    let (items, page_token) = get_subscriptions(
        &state.client,
        &payload.id,
        &api_key,
        payload.page_token.as_deref(),
        MAX_RESULTS,
    ).await?;

    Ok(Json(SubscriptionsResponse {
        items,
        page_token,
    }))
}

async fn index_handler() -> Html<String> {
    let html_content = include_str!("../../static/index.html");
    Html(html_content.to_string())
}

pub fn create_router() -> Router {
    let client = reqwest::Client::new();
    let state = Arc::new(AppState { client });

    Router::new()
        .route("/", get(index_handler))  // Add this line for serving the HTML
        .route("/api/playlist_items", post(playlist_items_handler))
        .route("/api/subscriptions", post(subscriptions_handler))
        .route("/api/channel", post(channel_handler))
        .with_state(state)
}