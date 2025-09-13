use lapin::Error as LapinError;
use serde_json::Error as SerdeJsonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Lapin error: {0}")]
    LapinError(#[from] LapinError),

    #[error("Serde JSON error: {0}")]
    SerdeJsonError(#[from] SerdeJsonError),
}
