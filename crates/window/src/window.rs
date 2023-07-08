use std::collections::HashMap;

use anyhow::{bail, Context};
use log::info;
use winit::{
    dpi::PhysicalSize,
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Fullscreen, Window, WindowBuilder, WindowId},
};

use crate::error::WindowsError;

/// The window attributes provided to the created window by default.
pub struct WindowOptions {
    /// Title displayed on the window. Usually in the title bar.
    pub title: String,
    /// When `true`, the window would be displayed in full screen.
    pub full_screen: bool,
    /// Set whether the window is resizable.
    pub resizable: bool,
    /// The size of the window, width and height. The size would be capped to maximum logical size of the monitor where the app is displayed.
    pub size: PhysicalSize<u32>,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "oxdraw window".to_owned(),
            full_screen: false,
            resizable: true,
            // TODO(Quadri): default to value used internally by winit for default window
            size: PhysicalSize::new(400, 400),
        }
    }
}

/// `WindowManager` manages windows and the event loop for a desktop application.
#[derive(Default)]
pub struct WindowManager {
    windows: HashMap<WindowId, Window>,
    main_window: Option<WindowId>,
    focus_window: Option<WindowId>,
}

/// Constructor functions
impl WindowManager {
    /// Create an instance of the application main window.
    /// `WindowManager` does not create a window by default, create the default window via this method or the application will crash with no specified window error.
    /// Acts as the default parent window for any other created window.
    pub fn with_main_window(mut self, event_loop: &EventLoop<()>) -> anyhow::Result<WindowManager> {
        let monitor = &event_loop
            .available_monitors()
            .next()
            .with_context(|| "No monitor found!".to_owned())?;

        // Make the default app window half size of the monitor being used for display
        let PhysicalSize { width, height } = monitor.size();
        let window_width = (width as f32 * 0.5) as u32;
        let window_height = (height as f32 * 0.5) as u32;
        let app_size = PhysicalSize::new(window_width, window_height);
        let id = self.create_window(
            WindowOptions {
                title: "Oxdraw".to_owned(),
                size: app_size,
                ..Default::default()
            },
            event_loop,
        )?;
        self.main_window = Some(id);
        Ok(self)
    }
}

/// Reference functions
impl WindowManager {
    /// Get an `Option<Window>` of a window with the specified id
    pub fn window(&self, window_id: &WindowId) -> Option<&Window> {
        self.windows.get(window_id)
    }

    /// Get a reference to the main window
    pub fn main_window(&self) -> anyhow::Result<&Window> {
        let main_window_id = match self.main_window {
            None => bail!(WindowsError::MainWindowId),
            Some(id) => id,
        };

        match self.window(&main_window_id) {
            None => bail!(WindowsError::WindowStore(main_window_id)),
            Some(window) => Ok(window),
        }
    }

    /// Return `true` if the `id` of the current window
    pub fn is_current_window(&self, window_id: &WindowId) -> bool {
        self.focus_window
            .map_or(false, |focus_window_id| &focus_window_id == window_id)
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Retrieve the information of the current window, panics if cannot find the window.
    pub fn current_window(&self) -> anyhow::Result<&Window> {
        if let Some(window_id) = self.focus_window {
            match self.window(&window_id) {
                None => bail!(WindowsError::WindowStore(window_id)),
                Some(window) => return Ok(window),
            }
        }
        return self.main_window();
    }
}

/// Mutable functions
impl WindowManager {
    pub fn focus_window(&mut self, window_id: WindowId) {
        self.focus_window = Some(window_id);
    }

    pub fn remove(&mut self, window_id: &WindowId) {
        self.windows.remove(window_id);
    }

    /// Creates a new window and store a reference to the window in the list of windows managed by this manager
    pub fn create_window(
        &mut self,
        options: WindowOptions,
        event_loop: &EventLoopWindowTarget<()>,
    ) -> anyhow::Result<WindowId> {
        let mut builder = WindowBuilder::new();

        // Initialize the window as invisible until all state has been properly set up.
        // `winit` windows need to be invisible for the `AccessKit` adapter is initialized.
        builder = builder.with_visible(false);

        // Set the options for the window
        builder = builder.with_title(options.title);
        builder = builder.with_inner_size(options.size);
        if options.full_screen {
            let monitor = &event_loop
                .available_monitors()
                .next()
                .with_context(|| "No monitor found!".to_owned())?;
            let video_mode = monitor
                .video_modes()
                .next()
                .with_context(|| "Failed to get list of display modes")?;
            builder = builder.with_fullscreen(Some(Fullscreen::Exclusive(video_mode)));
        }
        builder = builder.with_resizable(options.resizable);

        // TODO(Quadri): Get raw_window_handle requires unsafe access atm, figure out how to access it in a safe mode.
        // Set the main window as the default parent window
        // if let Some(main_window_id) = self.main_window {
        //     match self.windows.get(&main_window_id) {
        //         None => bail!("Unable to get the main window: {:?}", main_window_id),
        //         Some(main_window) => {
        //             builder.with_parent_window(main_window.raw_window_handle());
        //         }
        //     }
        // }
        let window = builder
            .build(event_loop)
            .with_context(|| "Unable to create the window!".to_owned())?;

        let window_id = window.id();
        self.windows.insert(window_id, window);

        Ok(window_id)
    }

    /// Run the setup process for window managers
    pub fn run(&self) -> anyhow::Result<()> {
        info!("Begin app startup with main window!");
        // Check to ensure there is a default window to start the app
        self.main_window()?;
        Ok(())
    }
}
