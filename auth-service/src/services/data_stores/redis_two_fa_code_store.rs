use std::sync::Arc;

use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{
    data_stores::{ TwoFACodeStore, TwoFACodeStoreError}, 
    Email, LoginAttemptId, TwoFACode,
};

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self{ conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(&mut self, 
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode
    ) -> Result<(), TwoFACodeStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(&email);

        let two_fa_tuple = TwoFATuple(login_attempt_id.as_ref().to_owned(), code.as_ref().to_owned());

        let serialized_data = serde_json::to_string(&two_fa_tuple).map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let _:() = conn
            .set_ex(&key, serialized_data, TEN_MINUTES_IN_SECONDS)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(email);

        let _:() = conn
           .del(&key)
           .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(email);

        match conn.get::<_, String>(&key) {
            Ok(data) => {
                let two_fa_tuple: TwoFATuple = serde_json::from_str(&data)
                    .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
                
                Ok((LoginAttemptId::parse(two_fa_tuple.0).unwrap(), TwoFACode::parse(two_fa_tuple.1).unwrap()))
            },
            Err(_) => Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}