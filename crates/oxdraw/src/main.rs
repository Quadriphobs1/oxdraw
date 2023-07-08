//! The `oxdraw` binary application.
//! A virtual space for sketching hand drawing and whiteboards in rust
use app::App;
use window::Windows;

fn main() -> anyhow::Result<()> {
    // Setup the app level logger
    match cfg!(debug_assertions) {
        true => setup_logger(log::LevelFilter::Debug),
        false => setup_logger(log::LevelFilter::Info),
    }

    let wm = Windows::new()?;
    let app = App::default();

    app.render(())?;
    // launch the app with main window
    wm.run()?;
    Ok(())
}

/// Default logging configuration for the application.Defaults logging to `stdout`
fn setup_logger(level: log::LevelFilter) {
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .filter(None, level)
        // turn off logging for potential libraries that we are using
        .filter(Some("wgpu_core"), log::LevelFilter::Off)
        .filter(Some("wgpu_hal"), log::LevelFilter::Off)
        .filter(Some("naga"), log::LevelFilter::Off)
        .init();
}
