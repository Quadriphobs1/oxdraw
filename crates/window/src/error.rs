use std::str::Utf8Error;

use thiserror::Error;
use winit::window::WindowId;

#[derive(Debug, Error)]
pub enum WindowsError {
    #[error("Unable to change focus for the window id: {0:?}")]
    Focus(WindowId),
    #[error("No main window id available!")]
    MainWindowId,
    #[error("Unable to get window with window id: {0:?}")]
    WindowStore(WindowId),
    #[error("Unexpected output: {0:?}")]
    StringUtfError(#[from] Utf8Error),
    #[error("`{0}` failed: {1}\n\n{2}")]
    ProcessOutputError(String, String, String),
}
