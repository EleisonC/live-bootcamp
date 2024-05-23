use axum::{http::StatusCode, extract::State,
    response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, domain::{AuthAPIError, User}, services::UserStoreError};
#[derive(Deserialize)]
pub struct SignupRequest {
    pub email:String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SignupResponse {
    pub message: String
}

pub async fn signup(State(state): State<AppState>, Json(request): Json<SignupRequest>) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email;
    let password = request.password;

    if email.is_empty() || !email.contains("@") || password.trim().to_string().capacity() < 8 || email.trim().to_string().capacity() <= 1 {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let user = User::new(email, password, request.requires_2fa);

    let mut user_store = state.user_store.write().await;

    match user_store.add_user(user) {
        Ok(()) => {
            let response = Json(SignupResponse {
                message: "User created successfully!".to_string()
            });
            
            Ok((StatusCode::CREATED, response))
        },
        Err(UserStoreError::UserAlreadyExists) => Err(AuthAPIError::UserAlreadyExists),
        Err(_) => Err(AuthAPIError::UnexpectedError),
    }
}
