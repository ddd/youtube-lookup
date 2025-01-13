use serde::Serialize;

#[derive(PartialEq)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    None,
    Verified,
    OAC
}

#[derive(Debug, Clone, Serialize)]
pub struct Channel {
    pub user_id: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub handle: Option<String>,
    pub profile_picture: Option<String>,
    pub banner: Option<String>,
    pub created_at: i64,
    pub country: Option<String>,
    pub view_count: i64,
    pub subscriber_count: i64,
    pub video_count: i64,
    pub made_for_kids: bool,
    pub keywords: Option<Vec<String>>,
    pub trailer: Option<String>,
    pub analytics_account_id: Option<String>,

    // the following require innertube
    pub conditional_redirect: Option<String>,
    pub no_index: Option<bool>,
    pub verification: Option<VerificationStatus>,
    pub blocked_countries: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Video {
    pub video_id: String,
    pub title: String,
    pub description: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Subscription {
    pub channel_id: String,
    pub title: String,
    pub created_at: i64,
    pub profile_picture: Option<String>,
}