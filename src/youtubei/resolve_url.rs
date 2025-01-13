use serde::{Serialize, Deserialize};
use crate::errors::YouTubeError;

#[derive(Debug, Serialize)]
struct InnertubeClient {
    #[serde(rename = "clientName")]
    client_name: String,
    #[serde(rename = "clientVersion")]
    client_version: String,
}

#[derive(Debug, Serialize)]
struct InnertubeContext {
    client: InnertubeClient,
}

#[derive(Debug, Serialize)]
struct ResolveUrlRequest {
    context: InnertubeContext,
    url: String,
}

#[derive(Debug, Deserialize)]
struct BrowseEndpoint {
    #[serde(rename = "browseId")]
    browse_id: String,
}

#[derive(Debug, Deserialize)]
struct UrlEndpoint {
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EndpointType {
    Browse {
        #[serde(rename = "browseEndpoint")]
        browse_endpoint: BrowseEndpoint,
    },
    Url {
        #[serde(rename = "urlEndpoint")]
        url_endpoint: UrlEndpoint,
    },
}

#[derive(Debug, Deserialize)]
struct Response {
    endpoint: EndpointType,
}

#[derive(Debug)]
pub enum ResolveUrlResult {
    BrowseEndpoint {
        browse_id: String,
    },
    UrlEndpoint {
        url: String,
    },
}

pub async fn resolve_url(
    client: &reqwest::Client,
    url: String,
) -> Result<Option<ResolveUrlResult>, YouTubeError> {
    let request = ResolveUrlRequest {
        context: InnertubeContext {
            client: InnertubeClient {
                client_name: "WEB".to_string(),
                client_version: "2.20240101".to_string(),
            },
        },
        url,
    };

    let resp = client
        .post("https://www.youtube.com/youtubei/v1/navigation/resolve_url")
        .header("Host", "youtubei.googleapis.com")
        .header("X-Goog-Fieldmask", "endpoint(urlEndpoint.url,browseEndpoint.browseId)")
        .json(&request)
        .send()
        .await
        .map_err(|e| YouTubeError::Other(Box::new(e)))?;

    match resp.status() {
        reqwest::StatusCode::OK => (),
        reqwest::StatusCode::NOT_FOUND => return Err(YouTubeError::NotFound),
        reqwest::StatusCode::UNAUTHORIZED => return Err(YouTubeError::Unauthorized),
        status => {
            let body_str = resp
                .text()
                .await
                .map_err(|e| YouTubeError::ParseError(e.to_string()))?;
            eprintln!("Unknown status code {}: {}", status.as_u16(), body_str);
            return Err(YouTubeError::UnknownStatusCode(status.into()));
        }
    }

    let response: Response = resp
        .json()
        .await
        .map_err(|e| YouTubeError::ParseError(e.to_string()))?;

    match response.endpoint {
        EndpointType::Browse { browse_endpoint } => Ok(Some(ResolveUrlResult::BrowseEndpoint {
            browse_id: browse_endpoint.browse_id,
        })),
        EndpointType::Url { url_endpoint } => Ok(Some(ResolveUrlResult::UrlEndpoint {
            url: url_endpoint.url,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    #[tokio::test]
    async fn test_resolve_browse_endpoint() {
        let client = Client::new();
        let result = resolve_url(
            &client,
            "youtube.com/@YouTube".to_string(),
        ).await;

        match result {
            Ok(Some(ResolveUrlResult::BrowseEndpoint { browse_id })) => {
                assert_eq!(browse_id, "UCBR8-60-B28hp2BmDPdntcQ");
            }
            _ => panic!("Expected BrowseEndpoint result"),
        }
    }

    #[tokio::test]
    async fn test_resolve_url_endpoint() {
        let client = Client::new();
        let result = resolve_url(
            &client,
            "youtube.com/NikPMusic".to_string(),
        ).await;

        match result {
            Ok(Some(ResolveUrlResult::UrlEndpoint { url })) => {
                assert_eq!(url, "http://www.youtube.com/channel/UCtI6KR_Y7memgBmEW0p8POw");
            }
            _ => panic!("Expected UrlEndpoint result"),
        }
    }
}