use auth_service::{domain::{Email, TwoFACodeStore}, routes::TwoFactorAuthResponse, ErrorResponse};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn should_return_422_if_malformed_inout() {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let invalid_data = [
        serde_json::json!({
            "mail": random_email,
            "loginAttemtId": "string",
            "2FACode": "99999"
        }),
        serde_json::json!({
            "email": random_email,
            "login_attemt_id": "string",
            "2FACode": "99999"
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemtId": "string",
            "2_FA_Code": "99999"
        }),
        serde_json::json!({
            "e-mail": random_email,
            "login_Attemt_Id": "string",
            "2_FA_Code": "99999"
        }),
    ];

    for data in invalid_data.iter() {
        let response = app.verify_2fa(data).await;

        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            data
        )
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let invalid_data = [
        serde_json::json!({
            "email": "random_email.com",
            "loginAttemptId": "a3bed8ad-be5b-4641-859a-2078ff544d4c",
            "2FACode": "99999"
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": "a3bed8ad-be5b-4641-859a",
            "2FACode": "99999"
        }),
        serde_json::json!({
            "email": random_email,
            "loginAttemptId": "a3bed8ad-be5b-4641-859a-2078ff544d4c",
            "2FACode": "99999102"
        }),
    ];

    for data in invalid_data.iter() {
        let response = app.verify_2fa(data).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            data
        );

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
                "Invalid credentials".to_string()
        )
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let valid_signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.signup(&valid_signup_body).await;

    assert_eq!(
        response.status().as_u16(),
        201
    );

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123"
    });

    let response = app.login(&login_body).await;

    assert_eq!(
        response.status().as_u16(),
        206
    );

    let incorrect_data = serde_json::json!({
        "email": random_email,
        "loginAttemptId": "a3bed8ad-be5b-4641-859a-2078ff544d4c",
        "2FACode": "206132"
    });

    let response = app.verify_2fa(&incorrect_data).await;

    assert_eq!(
        response.status().as_u16(),
        401
    );

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
            "Incorrect credentials".to_string()
    )
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let valid_signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.signup(&valid_signup_body).await;

    assert_eq!(
        response.status().as_u16(),
        201
    );

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123"
    });

    let response = app.login(&login_body).await;

    assert_eq!(
        response.status().as_u16(),
        206
    );

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await.expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());
    assert!(!json_body.login_attempt_id.is_empty());

    let email = Email::parse(random_email.clone()).unwrap();
    let result = app.two_fa_code_store.read().await.get_code(&email).await.unwrap();

    let response = app.login(&login_body).await;

    assert_eq!(
        response.status().as_u16(),
        206
    );

    let code = result.1.as_ref();

    let incorrect_data = serde_json::json!({
        "email": random_email,
        "loginAttemptId": json_body.login_attempt_id,
        "2FACode": code
    });

    let response = app.verify_2fa(&incorrect_data).await;

    assert_eq!(
        response.status().as_u16(),
        401
    );

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
            "Incorrect credentials".to_string()
    )
}
