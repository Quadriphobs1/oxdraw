use thiserror::Error;

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("Unable to create device: {0}")]
    RendererDevice(String),
    #[error("Cannot find pixel buffer for window id: {0}, ensure to call render.setup() to initialize a pixel buffer for the window id")]
    MissingPixelBuffer(u64),
    #[error("Failed to render")]
    FailedRender(#[from] pixels::Error),
}
