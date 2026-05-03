use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct YandexErrorResponse {
    pub message: String,
    pub description: Option<String>,
}

#[derive(Debug)]
pub enum ApiError {
    Yandex(YandexErrorResponse),
    Network(String),
    Deserialization(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Yandex(e) => write!(
                f,
                "{} - {}",
                e.message,
                e.description.as_deref().unwrap_or("No info")
            ),
            Self::Network(s) => write!(f, "Network: {}", s),
            Self::Deserialization(s) => write!(f, "JSON: {}", s),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Conference {
    pub id: String,
    pub title: String,
    pub description: String,
    pub join_url: String,
    pub watch_url: String,
    pub cohosts: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct YandexUser {
    pub login: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cohost {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ConferenceRequest {
    pub live_stream: LiveStreamRequest,
    pub cohosts: Vec<Cohost>,
}

#[derive(Debug, Serialize)]
pub struct LiveStreamRequest {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct ConferenceDetails {
    pub id: String,
    pub join_url: String,
    pub live_stream: LiveStreamResponse,
}

#[derive(Debug, Deserialize)]
pub struct LiveStreamResponse {
    pub watch_url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CohostListResponse {
    pub cohosts: Vec<Cohost>,
}

#[derive(Default, Clone, PartialEq)]
pub enum View {
    #[default]
    Login,
    Main,
    Edit {
        index: Option<usize>,
        is_fetching: bool,
    },
}

#[derive(Default)]
pub struct AppState {
    pub current_view: View,
    pub conferences: Vec<Conference>,
    pub token: String,
    pub api_error: Option<String>,
    pub is_waiting: bool,
    pub edit_title: String,
    pub edit_description: String,
    pub edit_cohosts: String,
}

pub enum AppAction {
    Login {
        token: String,
    },
    Create {
        title: String,
        desc: String,
        cohosts: Vec<String>,
    },
    Update {
        id: String,
        index: usize,
        title: String,
        desc: String,
        cohosts: Vec<String>,
    },
    FetchDetails {
        id: String,
        index: usize,
    },
}

pub enum ApiResponse {
    LoginSuccess {
        token: String,
        login: String,
    },
    CreateSuccess(Conference),
    UpdateSuccess {
        index: usize,
        conference_partial: (String, ConferenceDetails, Vec<String>, String, String),
    },
    DetailsFetched {
        index: usize,
        details: ConferenceDetails,
        cohosts: Vec<Cohost>,
    },
    Error(String),
}
