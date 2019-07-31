use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct ServiceList {
    pub trusted: Vec<String>,
    pub untrusted: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Address {
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct PingResponse {
    pub ack: bool,
}
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct RegisterResponse {
    pub session: Option<Session>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Session {
    pub token: Uuid,
}

#[cfg(feature = "discover")]
pub mod discover {

    use super::{PingResponse, RegisterResponse};
    use reqwest;
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use std::thread::sleep;
    use std::time::Duration;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct ServiceIdentity {
        pub protocol: String,
        pub address: String,
    }

    pub fn get_peers(api_path: &str, protocol: &str) -> Result<super::ServiceList, reqwest::Error> {
        reqwest::get(&format!("{}/discover/{}", api_path, protocol))?.json()
    }

    #[derive(Debug)]
    pub enum RegisterError {
        RegisterRefused,
        RegisterExpired,
        RequestError(reqwest::Error),
    }

    impl std::fmt::Display for RegisterError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self {
                RegisterError::RegisterRefused => write!(f, "register refused"),
                RegisterError::RegisterExpired => write!(f, "register revoked by server"),
                RegisterError::RequestError(e) => write!(f, "request failed: {}", e),
            }
        }
    }
    impl std::error::Error for RegisterError {}
    impl From<reqwest::Error> for RegisterError {
        fn from(err: reqwest::Error) -> RegisterError {
            RegisterError::RequestError(err)
        }
    }

    pub fn register(api_path: &str, id: ServiceIdentity) -> Result<(), RegisterError> {
        let client = Client::new();
        let (address, protocol) = (
            super::Address {
                address: id.address,
            },
            id.protocol,
        );
        let session: RegisterResponse = client
            .post(&format!("{}/discover/{}", api_path, protocol))
            .json(&address)
            .send()?
            .json()?;
        let session = match session.session {
            Some(s) => s,
            None => return Err(RegisterError::RegisterRefused),
        };
        loop {
            let resp: PingResponse = client
                .put(&format!("{}/ping/{}", api_path, session.token))
                .send()?
                .json()?;
            if !resp.ack {
                return Err(RegisterError::RegisterExpired);
            }
            sleep(Duration::from_secs(50));
        }
    }
}
