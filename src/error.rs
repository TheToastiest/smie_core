use std::time::SystemTimeError;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug)]
pub enum SmieError {
    Redis(redis::RedisError),
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    SystemTime(SystemTimeError),
    Other(String),
}
impl std::error::Error for SmieError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SmieError::Redis(e)      => Some(e),
            SmieError::Sqlite(e)     => Some(e),
            SmieError::Io(e)         => Some(e),
            SmieError::SystemTime(e) => Some(e),
            SmieError::Other(_)      => None,
        }
    }
}

impl From<redis::RedisError> for SmieError {
    fn from(e: redis::RedisError) -> Self { SmieError::Redis(e) }
}
impl From<rusqlite::Error> for SmieError {
    fn from(e: rusqlite::Error) -> Self { SmieError::Sqlite(e) }
}
impl From<std::io::Error> for SmieError {
    fn from(e: std::io::Error) -> Self { SmieError::Io(e) }
}
impl From<SystemTimeError> for SmieError {
    fn from(e: SystemTimeError) -> Self { SmieError::SystemTime(e) }
}

impl Display for SmieError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            SmieError::Redis(e) => write!(f, "Redis error: {}", e),
            SmieError::Sqlite(e) => write!(f, "SQLite error: {}", e),
            SmieError::Io(e) => write!(f, "IO error: {}", e),
            SmieError::SystemTime(e) => write!(f, "SystemTime error: {}", e),
            SmieError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}