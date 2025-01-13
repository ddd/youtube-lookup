use reqwest::Client;
use serde::Deserialize;
use chrono::DateTime;
use crate::models::Subscription;
use crate::errors::YouTubeError;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    items: Option<Vec<ApiSubscription>>
}

#[derive(Debug, Deserialize)]
struct ApiSubscription {
    snippet: Option<SubscriptionSnippet>,
}

#[derive(Debug, Deserialize)]
struct ResourceId {
    #[serde(rename = "channelId")]
    channel_id: Option<String>
}

#[derive(Debug, Deserialize)]
struct SubscriptionSnippet {
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    title: Option<String>,
    #[serde(rename = "resourceId")]
    resource_id: ResourceId,
    thumbnails: Option<Thumbnails>,
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
struct ErrorResponse {
    error: Error,
}

#[derive(Debug, Deserialize)]
struct Error {
    message: String,
}

pub async fn get_subscriptions(
    client: &Client,
    channel_id: &str,
    api_key: &str,
    page_token: Option<&str>,
    max_results: u32,
) -> Result<(Vec<Subscription>, Option<String>), YouTubeError> {
    let mut url = format!(
        "https://youtube.googleapis.com/youtube/v3/subscriptions?channelId={}&part=snippet&order=alphabetical&maxResults={}",
        channel_id,
        max_results
    );

    if let Some(token) = page_token {
        url.push_str(&format!("&pageToken={}", token));
    }

    let mut request = client
        .get(&url)
        .header("Host", "youtube.googleapis.com")
        .header("X-Goog-Fieldmask", "nextPageToken,items(snippet(publishedAt,title,resourceId.channelId,thumbnails.default.url))");

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
            
            match error_response.error.message.as_str() {
                "Subscriptions could not be retrieved because the subscriber's account is closed." => {
                    return Err(YouTubeError::AccountClosed)
                },
                "Subscriptions could not be retrieved because the subscriber's account is suspended." => {
                    return Err(YouTubeError::AccountTerminated)
                },
                "The requester is not allowed to access the requested subscriptions." => {
                    return Err(YouTubeError::SubscriptionsPrivate)
                },
                msg if msg.starts_with("The request cannot be completed because you have exceeded your") => {
                    return Err(YouTubeError::Ratelimited)
                },
                _ => {
                    eprintln!("Unknown forbidden error message: {}", error_response.error.message);
                    return Err(YouTubeError::Forbidden)
                }
            }
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

    let subscriptions = api_response.items
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            let snippet = item.snippet?;
            let channel_id = snippet.resource_id.channel_id?;
            let title = snippet.title?;
            let created_at = snippet.published_at?;
            
            let timestamp = DateTime::parse_from_rfc3339(&created_at)
                .ok()?
                .timestamp();

            let profile_picture = snippet
                .thumbnails
                .and_then(|t| t.default)
                .and_then(|d| d.url)
                .map(|avatar_url| {
                    avatar_url
                        .strip_prefix("https://yt3.ggpht.com/")
                        .unwrap_or(&avatar_url)
                        .split('=')
                        .next()
                        .unwrap_or(&avatar_url)
                        .to_string()
                });

            Some(Subscription {
                channel_id,
                title,
                created_at: timestamp,
                profile_picture,
            })
        })
        .collect();

    Ok((subscriptions, api_response.next_page_token))
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
    async fn test_get_subscriptions() {
        let client = Client::new();
        let result = get_subscriptions(
            &client,
            "UCewMTclBJZPaNEfbf-qYMGA",
            &get_api_key(),
            None,
            5,
        ).await;

        match result {
            Ok((subscriptions, next_page_token)) => {
                assert_eq!(subscriptions.len(), 5);
                assert_eq!(next_page_token, Some("CAUQAA".to_string()));

                let sub = &subscriptions[0];
                assert_eq!(sub.channel_id, "UCewMTclBJZPaNEfbf-qYMGA");
                assert_eq!(sub.title, "unstopble gameing");
                // 2024-03-04T10:27:40.003572Z as timestamp
                assert_eq!(sub.created_at, 1709548060);
            }
            Err(e) => panic!("Expected successful response, got error: {:?}", e),
        }
    }
}