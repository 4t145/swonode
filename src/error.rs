use std::{borrow::Cow, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ErrorCode {
    Unready = 0,
    LoopDetected = 10000,
    NotFound = 10001,
    Unreachable = 10002,
    Invalid = 10003,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    pub message: ErrorMessage,
    pub code: ErrorCode,
    pub traceback: Vec<Error>,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)?;
        f.write_str(&format!(
            "[{code:?}]: {message}",
            code = &self.code,
            message = self.message
        ))?;
        Ok(())
    }
}

impl Error {
    pub fn from_message(message: impl Into<ErrorMessage>, code: ErrorCode) -> Self {
        Self {
            message: message.into(),
            code,
            traceback: Default::default(),
        }
    }
    pub fn from_error(error: impl std::error::Error, code: ErrorCode) -> Self {
        Self {
            message: error.to_string().into(),
            code,
            traceback: Default::default(),
        }
    }
    pub fn code<E: std::error::Error>(code: ErrorCode) -> impl Fn(E) -> Self {
        move |e: E| Self::from_error(e, code)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

type ErrorMessage = Cow<'static, str>;
pub fn error<T>(message: impl Into<ErrorMessage>, code: ErrorCode) -> Result<T> {
    Result::Err(Error::from_message(message, code))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Response<T> {
    Ok(T),
    Err(Error),
}
