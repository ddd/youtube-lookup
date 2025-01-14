use reqwest::Client;
use serde::Deserialize;
use crate::models::Video;
use crate::errors::YouTubeError;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    items: Option<Vec<ApiVideo>>
}

#[derive(Debug, Deserialize)]
struct ApiVideo {
    id: String,
    statistics: Option<Statistics>,
    #[serde(rename = "liveStreamingDetails")]
    live_streaming_details: Option<LiveStreamingDetails>
}

#[derive(Debug, Deserialize)]
struct Statistics {
    #[serde(rename = "viewCount")]
    view_count: Option<String>,
    #[serde(rename = "likeCount")]
    like_count: Option<String>,
    #[serde(rename = "commentCount")]
    comment_count: Option<String>
}

#[derive(Debug, Deserialize)]
struct LiveStreamingDetails {
    #[serde(rename = "actualStartTime")]
    actual_start_time: Option<String>,
    #[serde(rename = "concurrentViewers")]
    concurrent_viewers: Option<String>
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Error,
}

#[derive(Debug, Deserialize)]
struct Error {
    message: String,
}

pub async fn populate_video_stats(
    client: &Client,
    videos: &mut Vec<Video>,
    api_key: &str,
) -> Result<(), YouTubeError> {
    // If no videos, return early
    if videos.is_empty() {
        return Ok(());
    }

    // Collect video IDs into owned strings
    let video_ids: Vec<String> = videos.iter().map(|v| v.video_id.clone()).collect();

    // Create chunks of 50 videos (YouTube API limit)
    for chunk in video_ids.chunks(50) {
        let ids = chunk.join(",");
        let url = format!(
            "https://youtube.googleapis.com/youtube/v3/videos?id={}&part=liveStreamingDetails,statistics",
            ids
        );

        let request = client
            .get(&url)
            .header("Host", "youtube.googleapis.com")
            .header("X-Goog-Api-Key", api_key)
            .header("X-Goog-Fieldmask", "items(id,statistics(viewCount,likeCount,commentCount),liveStreamingDetails(actualStartTime,concurrentViewers))");

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

        // Create a map of video stats
        let stats_map: std::collections::HashMap<String, (bool, Option<i64>, Option<i64>, Option<i64>)> = 
            api_response.items
                .unwrap_or_default()
                .into_iter()
                .map(|api_video| {
                    let is_livestream = api_video.live_streaming_details.is_some();
                    
                    let views = api_video.statistics
                        .as_ref()
                        .and_then(|s| s.view_count.as_ref())
                        .and_then(|v| v.parse::<i64>().ok());
                        
                    let likes = api_video.statistics
                        .as_ref()
                        .and_then(|s| s.like_count.as_ref())
                        .and_then(|v| v.parse::<i64>().ok());
                        
                    let comments = api_video.statistics
                        .as_ref()
                        .and_then(|s| s.comment_count.as_ref())
                        .and_then(|v| v.parse::<i64>().ok());

                    (api_video.id, (is_livestream, views, likes, comments))
                })
                .collect();

        // Update the video objects with the stats
        for video in videos.iter_mut() {
            if let Some((is_livestream, views, likes, comments)) = stats_map.get(&video.video_id) {
                video.livestream = *is_livestream;
                video.views = *views;
                video.likes = *likes;
                video.comments = *comments;
            }
        }
    }

    Ok(())
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
    async fn test_populate_video_stats() {
        let client = Client::new();
        let mut videos = vec![
            Video {
                video_id: "jfKfPfyJRdk".to_string(),
                title: "lofi hip hop radio 📚 - beats to relax/study to".to_string(),
                description: "".to_string(),
                livestream: false,
                views: None,
                likes: None,
                comments: None,
                created_at: 1657641570
            },
            Video {
                video_id: "SM66GDRyIVY".to_string(),
                title: "Turning Red | Official Trailer".to_string(),
                description: "".to_string(),
                livestream: false,
                views: None,
                likes: None,
                comments: None,
                created_at: 1643673600
            }
        ];

        let result = populate_video_stats(&client, &mut videos, &get_api_key()).await;
        assert!(result.is_ok());

        let first_video = &videos[0];
        assert!(first_video.livestream);
        assert!(first_video.views.is_some());
        assert!(first_video.likes.is_some());
        
        let second_video = &videos[1];
        assert!(!second_video.livestream);
        assert!(second_video.views.is_some());
        assert!(second_video.likes.is_some());
        assert!(second_video.comments.is_some());
    }
}