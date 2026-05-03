use crate::models::*;

const TELEMOST_BASE_URL: &str = "https://cloud-api.yandex.net/v1/telemost-api";

macro_rules! handle_res {
    ($res:expr) => {{
        let mut response = $res.map_err(|e| ApiError::Network(e.to_string()))?;
        let status = response.status();

        if status.is_success() {
            response
                .body_mut()
                .read_json::<T>()
                .map_err(|e| ApiError::Deserialization(e.to_string()))
        } else {
            match response.body_mut().read_json::<YandexErrorResponse>() {
                Ok(y_err) => Err(ApiError::Yandex(y_err)),
                Err(_) => Err(ApiError::Network(format!("API returned status {}", status))),
            }
        }
    }};
}

pub fn fetch_user_info(token: &str) -> Result<YandexUser, ApiError> {
    type T = YandexUser;
    handle_res!(
        ureq::get("https://login.yandex.ru/info")
            .header("Authorization", &format!("OAuth {}", token))
            .call()
    )
}

pub fn create_conference(
    token: &str,
    title: &str,
    desc: &str,
    cohosts: Vec<String>,
) -> Result<ConferenceDetails, ApiError> {
    type T = ConferenceDetails;
    let payload = ConferenceRequest {
        live_stream: LiveStreamRequest {
            title: title.to_string(),
            description: desc.to_string(),
        },
        cohosts: cohosts.into_iter().map(|email| Cohost { email }).collect(),
    };

    handle_res!(
        ureq::post(&format!("{}/conferences", TELEMOST_BASE_URL))
            .header("Authorization", &format!("OAuth {}", token))
            .send_json(payload)
    )
}

pub fn read_conference(token: &str, id: &str) -> Result<ConferenceDetails, ApiError> {
    type T = ConferenceDetails;
    handle_res!(
        ureq::get(&format!("{}/conferences/{}", TELEMOST_BASE_URL, id))
            .header("Authorization", &format!("OAuth {}", token))
            .call()
    )
}

pub fn read_cohosts(token: &str, id: &str) -> Result<Vec<Cohost>, ApiError> {
    type T = CohostListResponse;
    let res: CohostListResponse = handle_res!(
        ureq::get(&format!("{}/conferences/{}/cohosts", TELEMOST_BASE_URL, id))
            .header("Authorization", &format!("OAuth {}", token))
            .call()
    )?;

    Ok(res.cohosts)
}

pub fn update_conference(
    token: &str,
    id: &str,
    title: &str,
    desc: &str,
    cohosts: Vec<String>,
) -> Result<ConferenceDetails, ApiError> {
    type T = ConferenceDetails;
    let payload = ConferenceRequest {
        live_stream: LiveStreamRequest {
            title: title.to_string(),
            description: desc.to_string(),
        },
        cohosts: cohosts.into_iter().map(|email| Cohost { email }).collect(),
    };

    handle_res!(
        ureq::patch(&format!("{}/conferences/{}", TELEMOST_BASE_URL, id))
            .header("Authorization", &format!("OAuth {}", token))
            .send_json(payload)
    )
}
