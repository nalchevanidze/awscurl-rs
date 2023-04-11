use sha256::digest;
use std::{env, fmt::Display};

#[derive(Clone)]
pub enum Body {
    Empty,
    String(String),
    Binary(Vec<u8>),
}

#[derive(Clone)]
pub enum Method {
    GET,
    POST(Body),
    DELETE,
}

impl Method {
    pub fn hash_body(&self) -> String {
        match self {
            Method::POST(body) => match body {
                Body::Empty => digest(""),
                Body::String(txt) => digest(txt.to_string()),
                Body::Binary(bytes) => digest(&bytes[..]),
            },
            _ => digest(""),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::GET => write!(f, "GET"),
            Method::POST(_) => write!(f, "POST"),
            Method::DELETE => write!(f, "DELETE"),
        }
    }
}

#[derive(Clone)]
pub struct AWSProfile {
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub session_token: String,
}

fn env_var(key: &str) -> AWSIotResult<String> {
    env::var(key).map_err(|_| AWSIotError::new(&format!("cant found env variable: {}!", key)))
}

impl AWSProfile {
    pub fn from_env() -> AWSIotResult<AWSProfile> {
        let access_key = env_var("AWS_ACCESS_KEY_ID")?;
        let secret_key = env_var("AWS_SECRET_ACCESS_KEY")?;
        let region = env_var("AWS_REGION")?;
        let session_token = env_var("AWS_SESSION_TOKEN")?;

        Ok(AWSProfile {
            access_key,
            secret_key,
            region,
            session_token,
        })
    }
}

pub type AWSIotResult<T> = Result<T, AWSIotError>;

#[derive(Debug)]
pub struct AWSIotError {
    pub message: String,
}

impl AWSIotError {
    pub fn new<T: Display>(message: T) -> Self {
        AWSIotError {
            message: message.to_string(),
        }
    }
}

impl Display for AWSIotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
