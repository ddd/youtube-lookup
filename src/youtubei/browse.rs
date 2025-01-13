use serde::{Serialize, Deserialize};
use reqwest::Client;
use std::collections::HashSet;
use crate::models::Channel;
use crate::models::VerificationStatus;
use crate::errors::YouTubeError;

const ALL_COUNTRIES: &[&str] = &[
    "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX", "AZ",
    "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL", "BM", "BN", "BO", "BQ", "BR", "BS",
    "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK", "CL", "CM", "CN",
    "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM", "DO", "DZ", "EC", "EE",
    "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK", "FM", "FO", "FR", "GA", "GB", "GD", "GE", "GF",
    "GG", "GH", "GI", "GL", "GM", "GN", "GP", "GQ", "GR", "GS", "GT", "GU", "GW", "GY", "HK", "HM",
    "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN", "IO", "IQ", "IR", "IS", "IT", "JE", "JM",
    "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN", "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC",
    "LI", "LK", "LR", "LS", "LT", "LU", "LV", "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK",
    "ML", "MM", "MN", "MO", "MP", "MQ", "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA",
    "NC", "NE", "NF", "NG", "NI", "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG",
    "PH", "PK", "PL", "PM", "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW",
    "SA", "SB", "SC", "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SR", "SS",
    "ST", "SV", "SX", "SY", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO",
    "TR", "TT", "TV", "TW", "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI",
    "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW"
];

#[derive(Debug, Deserialize, Clone, Serialize)]
struct InnertubeClient {
    #[serde(rename = "clientName")]
    client_name: String,
    #[serde(rename = "clientVersion")]
    client_version: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct InnertubeContext {
    client: InnertubeClient,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct BrowseRequest {
    context: InnertubeContext,
    #[serde(rename = "browseId")]
    browse_id: String,
}

#[derive(Debug, Deserialize, Clone)]
struct ImageSource {
    #[serde(rename = "clientResource")]
    client_resource: ClientResource,
}

#[derive(Debug, Deserialize, Clone)]
struct ClientResource {
    #[serde(rename = "imageName")]
    image_name: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Image {
    sources: Vec<ImageSource>,
}

#[derive(Debug, Deserialize, Clone)]
struct ImageType {
    image: Image,
}

#[derive(Debug, Deserialize, Clone)]
struct ElementType {
    #[serde(rename = "imageType")]
    image_type: ImageType,
}

#[derive(Debug, Deserialize, Clone)]
struct Element {
    r#type: ElementType,
}

#[derive(Debug, Deserialize, Clone)]
struct AttachmentRun {
    element: Element,
}

#[derive(Debug, Deserialize, Clone)]
struct Text {
    #[serde(rename = "attachmentRuns")]
    attachment_runs: Option<Vec<AttachmentRun>>,
}

#[derive(Debug, Deserialize, Clone)]
struct DynamicTextViewModel {
    text: Text,
}

#[derive(Debug, Deserialize, Clone)]
struct Title {
    #[serde(rename = "dynamicTextViewModel")]
    dynamic_text_view_model: DynamicTextViewModel,
}

#[derive(Debug, Deserialize, Clone)]
struct PageHeaderViewModel {
    title: Title,
}

#[derive(Debug, Deserialize, Clone)]
struct Content {
    #[serde(rename = "pageHeaderViewModel")]
    page_header_view_model: PageHeaderViewModel,
}

#[derive(Debug, Deserialize, Clone)]
struct PageHeaderRenderer {
    content: Content,
}

#[derive(Debug, Deserialize, Clone)]
struct Header {
    #[serde(rename = "pageHeaderRenderer")]
    page_header_renderer: PageHeaderRenderer,
}

#[derive(Debug, Deserialize, Clone)]
struct BrowseEndpoint {
    #[serde(rename = "browseId")]
    browse_id: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Endpoint {
    #[serde(rename = "browseEndpoint")]
    browse_endpoint: BrowseEndpoint,
}

#[derive(Debug, Deserialize, Clone)]
struct NavigateAction {
    endpoint: Endpoint,
}

#[derive(Debug, Deserialize, Clone)]
struct OnResponseReceivedAction {
    #[serde(rename = "navigateAction")]
    navigate_action: NavigateAction,
}

#[derive(Debug, Deserialize, Clone)]
struct MicroformatDataRenderer {
    noindex: Option<bool>,
    #[serde(rename = "availableCountries")]
    available_countries: Option<Vec<String>>
}

#[derive(Debug, Deserialize, Clone)]
struct Microformat {
    #[serde(rename = "microformatDataRenderer")]
    microformat_data_renderer: MicroformatDataRenderer,
}

#[derive(Debug, Deserialize, Clone)]
struct ChannelMetadataRenderer {
    #[serde(rename = "ownerUrls")]
    owner_urls: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
struct Metadata {
    #[serde(rename = "channelMetadataRenderer")]
    channel_metadata_renderer: ChannelMetadataRenderer,
}

#[derive(Debug, Deserialize, Clone)]
struct BrowseResponse {
    header: Option<Header>,
    #[serde(rename = "onResponseReceivedActions")]
    on_response_received_actions: Option<Vec<OnResponseReceivedAction>>,
    microformat: Option<Microformat>,
    metadata: Option<Metadata>,
}

pub async fn enrich_channel_data(
    client: &Client,
    channel: &mut Channel,
) -> Result<(), YouTubeError> {
    let request = BrowseRequest {
        context: InnertubeContext {
            client: InnertubeClient {
                client_name: "WEB".to_string(),
                client_version: "2.20250108.06.00".to_string(),
            },
        },
        browse_id: channel.user_id.clone(),
    };

    let resp = client
        .post("https://www.youtube.com/youtubei/v1/browse?prettyPrint=false")
        .header("Host", "www.youtube.com")
        .header("Content-Type", "application/json")
        .header("X-Goog-Fieldmask", "onResponseReceivedActions.navigateAction.endpoint.browseEndpoint.browseId,header.pageHeaderRenderer.content.pageHeaderViewModel.title.dynamicTextViewModel.text.attachmentRuns.element.type.imageType.image.sources.clientResource.imageName,metadata.channelMetadataRenderer.ownerUrls,microformat.microformatDataRenderer(noindex,availableCountries)")
        .json(&request)
        .send()
        .await
        .map_err(|e| YouTubeError::Other(Box::new(e)))?;

    match resp.status() {
        reqwest::StatusCode::OK => {
            let response: BrowseResponse = resp
                .json()
                .await
                .map_err(|e| YouTubeError::ParseError(e.to_string()))?;

            // Handle conditional redirect
            if let Some(actions) = response.on_response_received_actions {
                if let Some(action) = actions.first() {
                    let redirect_id = action.navigate_action.endpoint.browse_endpoint.browse_id.clone();
                    if redirect_id != channel.user_id {
                        channel.conditional_redirect = Some(redirect_id);
                        return Ok(());
                    }
                }
            }

            // Parse verification status from badge
            if let Some(header) = response.header {
                let badge = header
                    .page_header_renderer
                    .content
                    .page_header_view_model
                    .title
                    .dynamic_text_view_model
                    .text
                    .attachment_runs
                    .and_then(|runs| runs.first().cloned())
                    .map(|run| run.element.r#type.image_type.image.sources[0].client_resource.image_name.clone());

                channel.verification = Some(match badge.as_deref() {
                    Some("AUDIO_BADGE") => VerificationStatus::OAC,
                    Some("CHECK_CIRCLE_FILLED") => VerificationStatus::Verified,
                    _ => VerificationStatus::None,
                });
            }

            // Parse microformat data
            if let Some(microformat) = response.microformat {
                channel.no_index = microformat.microformat_data_renderer.noindex;
                
                // Handle available countries
                if let Some(available) = microformat.microformat_data_renderer.available_countries {
                    let available: HashSet<_> = available.iter().cloned().collect();
                    let all: HashSet<_> = ALL_COUNTRIES.iter().map(|&s| s.to_string()).collect();
                    
                    // Countries that are not in the available list are blocked
                    let blocked: Vec<_> = all.difference(&available).cloned().collect();
                    channel.blocked_countries = if blocked.is_empty() { None } else { Some(blocked) };
                }
            }

            if let Some(metadata) = &response.metadata {
                if let Some(owner_urls) = &metadata.channel_metadata_renderer.owner_urls {
                    for url in owner_urls {
                        if let Some(handle) = url.strip_prefix("http://www.youtube.com/@") {
                            channel.handle = Some(handle.to_string());
                            break;
                        }
                    }
                }
            }

            Ok(())
        }
        status => {
            eprintln!("Unexpected status code: {}", status);
            Err(YouTubeError::UnknownStatusCode(status))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use crate::models::Channel;
    
    #[tokio::test]
    async fn test_channel_with_blocked_us() {
        let client = Client::new();
        let mut channel = Channel {
            user_id: "UC7A4ikI7Q4Efju5WNRdIqyg".to_string(),
            display_name: None,
            description: None,
            handle: None,
            profile_picture: None,
            banner: None,
            created_at: 0,
            country: None,
            view_count: 0,
            subscriber_count: 0,
            video_count: 0,
            made_for_kids: false,
            keywords: None,
            trailer: None,
            analytics_account_id: None,
            conditional_redirect: None,
            no_index: None,
            verification: None,
            blocked_countries: None,
        };

        let result = enrich_channel_data(&client, &mut channel).await;
        assert!(result.is_ok());
        
        assert_eq!(channel.verification, Some(VerificationStatus::None));
        assert_eq!(channel.conditional_redirect, None);
        assert_eq!(channel.no_index, Some(false));
        
        // Check that US is in blocked countries
        if let Some(blocked) = channel.blocked_countries {
            assert!(blocked.contains(&"US".to_string()));
        } else {
            panic!("Blocked countries should be populated");
        }
    }

    #[tokio::test]
    async fn test_verified_channel() {
        let client = Client::new();
        let mut channel = Channel {
            user_id: "UCewMTclBJZPaNEfbf-qYMGA".to_string(),
            display_name: None,
            description: None,
            handle: None,
            profile_picture: None,
            banner: None,
            created_at: 0,
            country: None,
            view_count: 0,
            subscriber_count: 0,
            video_count: 0,
            made_for_kids: false,
            keywords: None,
            trailer: None,
            analytics_account_id: None,
            conditional_redirect: None,
            no_index: None,
            verification: None,
            blocked_countries: None,
        };

        let result = enrich_channel_data(&client, &mut channel).await;
        assert!(result.is_ok());
        
        assert_eq!(channel.verification, Some(VerificationStatus::Verified));
        assert_eq!(channel.conditional_redirect, None);
        assert_eq!(channel.no_index, Some(false));
    }

    #[tokio::test]
    async fn test_artist_channel() {
        let client = Client::new();
        let mut channel = Channel {
            user_id: "UCsRM0YB_dabtEPGPTKo-gcw".to_string(),
            display_name: None,
            description: None,
            handle: None,
            profile_picture: None,
            banner: None,
            created_at: 0,
            country: None,
            view_count: 0,
            subscriber_count: 0,
            video_count: 0,
            made_for_kids: false,
            keywords: None,
            trailer: None,
            analytics_account_id: None,
            conditional_redirect: None,
            no_index: None,
            verification: None,
            blocked_countries: None,
        };

        let result = enrich_channel_data(&client, &mut channel).await;
        assert!(result.is_ok());
        
        assert_eq!(channel.verification, Some(VerificationStatus::OAC));
        assert_eq!(channel.conditional_redirect, None);
        assert_eq!(channel.no_index, Some(false));
    }

    #[tokio::test]
    async fn test_channel_with_redirect() {
        let client = Client::new();
        let mut channel = Channel {
            user_id: "UC80zzW0g4xuUwW6IffjhcDQ".to_string(),
            display_name: None,
            description: None,
            handle: None,
            profile_picture: None,
            banner: None,
            created_at: 0,
            country: None,
            view_count: 0,
            subscriber_count: 0,
            video_count: 0,
            made_for_kids: false,
            keywords: None,
            trailer: None,
            analytics_account_id: None,
            conditional_redirect: None,
            no_index: None,
            verification: None,
            blocked_countries: None,
        };

        let result = enrich_channel_data(&client, &mut channel).await;
        assert!(result.is_ok());
        
        assert!(channel.conditional_redirect.is_some());
    }

    #[tokio::test]
    async fn test_channel_with_noindex() {
        let client = Client::new();
        let mut channel = Channel {
            user_id: "UC-8U_MhAZnBXKZvI5kMllLA".to_string(),
            display_name: None,
            description: None,
            handle: None,
            profile_picture: None,
            banner: None,
            created_at: 0,
            country: None,
            view_count: 0,
            subscriber_count: 0,
            video_count: 0,
            made_for_kids: false,
            keywords: None,
            trailer: None,
            analytics_account_id: None,
            conditional_redirect: None,
            no_index: None,
            verification: None,
            blocked_countries: None,
        };

        let result = enrich_channel_data(&client, &mut channel).await;
        assert!(result.is_ok());
        
        assert_eq!(channel.no_index, Some(true));
    }
}