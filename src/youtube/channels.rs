use reqwest::Client;
use serde::Deserialize;
use chrono::DateTime;
use crate::models::Channel;
use crate::errors::YouTubeError;

#[derive(Debug)]
pub enum LookupType {
    Username(String),
    Handle(String),
    ChannelID(String)
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    items: Option<Vec<ApiChannel>>
}

#[derive(Debug, Deserialize)]
struct ApiChannel {
    id: String,
    snippet: Option<ChannelSnippet>,
    statistics: Option<ChannelStatistics>,
    status: Option<ChannelStatus>,
    #[serde(rename = "brandingSettings")]
    branding_settings: Option<BrandingSettings>
}

#[derive(Debug, Deserialize)]
struct ChannelSnippet {
    title: Option<String>,
    description: Option<String>,
    #[serde(rename = "customUrl")]
    custom_url: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    thumbnails: Option<Thumbnails>,
    country: Option<String>
}

#[derive(Debug, Deserialize)]
struct Thumbnails {
    default: Option<Thumbnail>
}

#[derive(Debug, Deserialize)]
struct Thumbnail {
    url: Option<String>
}

#[derive(Debug, Deserialize)]
struct ChannelStatistics {
    #[serde(rename = "viewCount")]
    view_count: Option<String>,
    #[serde(rename = "subscriberCount")]
    subscriber_count: Option<String>,
    #[serde(rename = "videoCount")]
    video_count: Option<String>
}

#[derive(Debug, Deserialize)]
struct ChannelStatus {
    #[serde(rename = "madeForKids")]
    made_for_kids: Option<bool>
}

#[derive(Debug, Deserialize)]
struct BrandingSettings {
    channel: Option<ChannelBranding>,
    image: Option<ChannelImage>
}

#[derive(Debug, Deserialize)]
struct ChannelBranding {
    keywords: Option<String>,
    #[serde(rename = "trackingAnalyticsAccountId")]
    tracking_analytics_account_id: Option<String>,
    #[serde(rename = "unsubscribedTrailer")]
    unsubscribed_trailer: Option<String>
}

#[derive(Debug, Deserialize)]
struct ChannelImage {
    #[serde(rename = "bannerExternalUrl")]
    banner_external_url: Option<String>
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Error,
}

#[derive(Debug, Deserialize)]
struct Error {
    message: String,
}

pub async fn get_channel(
    client: &Client,
    lookup_type: LookupType,
    api_key: &str,
) -> Result<Channel, YouTubeError> {
    let url = match &lookup_type {
        LookupType::Username(username) => format!(
            "https://youtube.googleapis.com/youtube/v3/channels?part=brandingSettings,id,snippet,statistics,status,localizations,topicDetails&forUsername={}",
            username
        ),
        LookupType::Handle(handle) => format!(
            "https://youtube.googleapis.com/youtube/v3/channels?part=brandingSettings,id,snippet,statistics,status,localizations,topicDetails&forHandle={}",
            handle
        ),
        LookupType::ChannelID(channel_id) => format!(
            "https://youtube.googleapis.com/youtube/v3/channels?part=brandingSettings,id,snippet,statistics,status,localizations,topicDetails&id={}",
            channel_id
        ),
    };

    let mut request = client
        .get(&url)
        .header("Host", "youtube.googleapis.com")
        .header("X-Goog-Fieldmask", "items(id,snippet(title,description,customUrl,publishedAt,country,thumbnails.default.url),statistics(subscriberCount,viewCount,videoCount),topicDetails.topicIds,brandingSettings(channel(keywords,unsubscribedTrailer,trackingAnalyticsAccountId),image.bannerExternalUrl),status.madeForKids)");

    request = request.header("X-Goog-Api-Key", api_key);

    let resp = request
        .send()
        .await
        .map_err(|e| YouTubeError::Other(Box::new(e)))?;

    match resp.status() {
        reqwest::StatusCode::TOO_MANY_REQUESTS => return Err(YouTubeError::Ratelimited),
        reqwest::StatusCode::FORBIDDEN => {
            let error_response: ErrorResponse = resp
                .json()
                .await
                .map_err(|e| YouTubeError::ParseError(e.to_string()))?;
            
            if error_response.error.message.starts_with("The request cannot be completed because you have exceeded your") {
                return Err(YouTubeError::Ratelimited);
            }
            return Err(YouTubeError::Forbidden);
        },
        reqwest::StatusCode::NOT_FOUND => return Err(YouTubeError::NotFound),
        reqwest::StatusCode::UNAUTHORIZED => return Err(YouTubeError::Unauthorized),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR | reqwest::StatusCode::SERVICE_UNAVAILABLE => {
            return Err(YouTubeError::InternalServerError);
        },
        reqwest::StatusCode::OK => (), // Continue processing
        status => {
            let body = resp
                .text()
                .await
                .map_err(|e| YouTubeError::ParseError(e.to_string()))?;
            eprintln!("Unknown status code {}: {}", status.as_u16(), body);
            return Err(YouTubeError::UnknownStatusCode(status));
        }
    }

    let api_response: ApiResponse = resp
        .json()
        .await
        .map_err(|e| YouTubeError::ParseError(e.to_string()))?;

    let channel = api_response.items
        .and_then(|mut items| items.pop())
        .ok_or(YouTubeError::NotFound)?;

    let profile_picture = channel.snippet
        .as_ref()
        .and_then(|s| s.thumbnails.as_ref())
        .and_then(|t| t.default.as_ref())
        .and_then(|d| d.url.as_ref())
        .map(|avatar_url| {
            avatar_url
                .strip_prefix("https://yt3.ggpht.com/")
                .unwrap_or(avatar_url)
                .split('=')
                .next()
                .unwrap_or(avatar_url)
                .to_string()
        });

    let banner = channel.branding_settings
        .as_ref()
        .and_then(|b| b.image.as_ref())
        .and_then(|i| i.banner_external_url.as_ref())
        .map(|banner_url| {
            // Try stripping both possible domains
            let stripped_url = banner_url
                .strip_prefix("https://yt3.googleusercontent.com/")
                .or_else(|| banner_url.strip_prefix("https://lh3.googleusercontent.com/"))
                .unwrap_or(banner_url);
            
            // Get everything before the first '=' if it exists
            stripped_url
                .split('=')
                .next()
                .unwrap_or(stripped_url)
                .to_string()
        });

    let handle = channel.snippet
        .as_ref()
        .and_then(|s| s.custom_url.as_ref())
        .and_then(|h| {
            if h.starts_with('@') {
                Some(h.trim_start_matches('@').to_string())
            } else {
                None
            }
        });

    Ok(Channel {
        user_id: channel.id,
        display_name: channel.snippet.as_ref().and_then(|s| s.title.clone()),
        description: channel.snippet.as_ref().and_then(|s| s.description.clone()),
        handle,
        profile_picture,
        banner,
        created_at: channel.snippet
            .as_ref()
            .and_then(|s| s.published_at.as_ref())
            .and_then(|dt| DateTime::parse_from_rfc3339(dt).ok())
            .map(|dt| dt.timestamp())
            .unwrap_or_default(),
        country: channel.snippet.as_ref().and_then(|s| s.country.clone()),
        view_count: channel.statistics
            .as_ref()
            .and_then(|s| s.view_count.as_ref())
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or_default(),
        subscriber_count: channel.statistics
            .as_ref()
            .and_then(|s| s.subscriber_count.as_ref())
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or_default(),
        video_count: channel.statistics
            .as_ref()
            .and_then(|s| s.video_count.as_ref())
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or_default(),
        made_for_kids: channel.status
            .and_then(|s| s.made_for_kids)
            .unwrap_or_default(),
        keywords: channel.branding_settings
            .as_ref()
            .and_then(|b| b.channel.as_ref())
            .and_then(|c| c.keywords.as_ref())
            .map(|k| {
                let mut tags = Vec::new();
                let mut current_tag = String::new();
                let mut in_quotes = false;
                
                for c in k.chars() {
                    match c {
                        '"' => {
                            in_quotes = !in_quotes;
                        },
                        ' ' if !in_quotes => {
                            if !current_tag.is_empty() {
                                tags.push(current_tag.trim().to_string());
                                current_tag.clear();
                            }
                        },
                        '\\' => (), // Skip escape characters
                        _ => current_tag.push(c),
                    }
                }
                
                if !current_tag.is_empty() {
                    tags.push(current_tag.trim().to_string());
                }
                
                tags.into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>()
            }),
        trailer: channel.branding_settings
            .as_ref()
            .and_then(|b| b.channel.as_ref())
            .and_then(|c| c.unsubscribed_trailer.clone()),
        analytics_account_id: channel.branding_settings
            .as_ref()
            .and_then(|b| b.channel.as_ref())
            .and_then(|c| c.tracking_analytics_account_id.clone()),
        blocked_countries: None,
        conditional_redirect: None,
        no_index: None,
        verification: None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    fn get_api_key() -> String {
        dotenvy::dotenv().ok();
        env::var("API_KEY").expect("API_KEY must be set")
    }

    #[tokio::test]
    async fn test_get_channel_by_id() {
        let client = Client::new();
        let result = get_channel(
            &client,
            LookupType::ChannelID("UCBR8-60-B28hp2BmDPdntcQ".to_string()),
            &get_api_key(),
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_channel_by_username() {
        let client = Client::new();
        let result = get_channel(
            &client,
            LookupType::Username("YouTube".to_string()),
            &get_api_key(),
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_channel_by_handle() {
        let client = Client::new();
        let result = get_channel(
            &client,
            LookupType::Handle("TeamYouTube".to_string()),
            &get_api_key(),
        ).await;

        assert!(result.is_ok());
    }
}