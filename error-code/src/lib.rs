use core::fmt;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    str::FromStr,
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

#[derive(Debug)]
pub struct ErrorInfo<T> {
    pub app_code: T,
    pub code: &'static str,
    pub hash: String,
    pub client_msg: &'static str,
    pub server_msg: String,
}

pub trait ToErrorInfo {
    type T: FromStr;
    fn to_error_info(&self) -> Result<ErrorInfo<Self::T>, <Self::T as FromStr>::Err>;
}

impl<T> ErrorInfo<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    pub fn try_new(
        app_code: &str,
        code: &'static str,
        client_msg: &'static str,
        server_msg: impl fmt::Display,
    ) -> Result<Self, T::Err> {
        let server_msg = server_msg.to_string();
        let mut hasher = DefaultHasher::new();
        server_msg.hash(&mut hasher);
        let hash = hasher.finish();
        let hash = URL_SAFE_NO_PAD.encode(hash.to_be_bytes());
        Ok(Self {
            app_code: T::from_str(app_code).expect("cannot convert app_code"),
            code,
            hash,
            client_msg,
            server_msg: server_msg.to_string(),
        })
    }
}
