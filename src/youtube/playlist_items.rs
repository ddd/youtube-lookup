use reqwest::Client;
use serde::Deserialize;
use chrono::DateTime;
use crate::models::Video;
use crate::errors::YouTubeError;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    items: Option<Vec<ApiPlaylistItem>>
}

#[derive(Debug, Deserialize)]
struct ApiPlaylistItem {
    snippet: Option<ItemSnippet>,
}

#[derive(Debug, Deserialize)]
struct ItemSnippet {
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    title: Option<String>,
    description: Option<String>,
    #[serde(rename = "resourceId")]
    resource_id: Option<ResourceId>,
}

#[derive(Debug, Deserialize)]
struct ResourceId {
    #[serde(rename = "videoId")]
    video_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Error,
}

#[derive(Debug, Deserialize)]
struct Error {
    message: String,
}

pub async fn get_playlist_items(
    client: &Client,
    playlist_id: &str,
    api_key: &str,
    page_token: Option<&str>,
    max_results: u32,
) -> Result<(Vec<Video>, Option<String>), YouTubeError> {
    let mut url = format!(
        "https://youtube.googleapis.com/youtube/v3/playlistItems?playlistId={}&part=snippet&maxResults={}",
        playlist_id,
        max_results
    );

    if let Some(token) = page_token {
        url.push_str(&format!("&pageToken={}", token));
    }

    let mut request = client
        .get(&url)
        .header("Host", "youtube.googleapis.com")
        .header("X-Goog-Fieldmask", "nextPageToken,items(snippet(publishedAt,title,description,resourceId.videoId))");

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

    let videos = api_response.items
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| {
            let snippet = item.snippet?;
            let video_id = snippet.resource_id?.video_id?;
            let title = snippet.title?;
            let description = snippet.description.unwrap_or_default();
            let created_at = snippet.published_at?;
            
            let timestamp = DateTime::parse_from_rfc3339(&created_at)
                .ok()?
                .timestamp();

            Some(Video {
                video_id,
                title,
                description,
                created_at: timestamp,
                livestream: false,
                views: None,
                likes: None,
                comments: None
            })
        })
        .collect();

    Ok((videos, api_response.next_page_token))
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
    async fn test_get_playlist_items() {
        let client = Client::new();
        let result = get_playlist_items(
            &client,
            "UUwBkSWEuckW8AHZ62XcSLYw",
            &get_api_key(),
            None,
            5,
        ).await;

        match result {
            Ok((videos, next_page_token)) => {
                assert_eq!(videos.len(), 5);
                assert!(next_page_token.is_some());

                let first_video = &videos[0];
                assert_eq!(first_video.video_id, "gfKpRpwHckY");
                assert_eq!(first_video.title, "First Video!!!");
                assert_eq!(first_video.description, "");
                // 2023-07-08T13:29:24Z as timestamp
                assert_eq!(first_video.created_at, 1688822964);
            }
            Err(e) => panic!("Expected successful response, got error: {:?}", e),
        }
    }
}