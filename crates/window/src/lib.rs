//! Desktop application window management library. A wrapper around [winit](https://github.com/rust-windowing/winit) for handling window management

mod context;
mod error;
mod event;
mod key_code;
mod menu;
mod run;
mod window;

use std::{cell::RefCell, rc::Rc, time::Instant};

use menu::MenuManager;
use renderer::Renderer;
use run::winit_runner;
use winit::event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy};

pub struct Windows {
    event_loop: Rc<RefCell<EventLoop<()>>>,
    wm: window::WindowManager,
    menu: MenuManager,
    ctx: context::Context,
    renderer: Renderer,
    state: WinitPersistentState,
}

impl Windows {
    pub fn new() -> anyhow::Result<Windows> {
        let mut event_loop_builder = EventLoopBuilder::new();

        let menu = MenuManager::new(&mut event_loop_builder);

        let event_loop = event_loop_builder.build();

        let event_loop = Rc::new(RefCell::new(event_loop));
        let wm = window::WindowManager::default().with_main_window(&event_loop.borrow_mut())?;
        let renderer = Renderer::new()?;
        let ctx = context::Context::default();
        let state = WinitPersistentState::default();
        Ok(Windows {
            event_loop,
            wm,
            menu,
            ctx,
            renderer,
            state,
        })
    }

    pub fn proxy_handler(&self) -> EventLoopProxy<()> {
        self.event_loop.borrow_mut().create_proxy()
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        self.wm.run()?;
        self.menu.setup()?;
        winit_runner(self);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowMode {
    Wait,
    WaitUntil,
    Poll,
}

/// Stores state persists between frames.
pub struct WinitPersistentState {
    /// Tracks whether or not the application is active or suspended.
    active: bool,
    /// Tracks whether the event loop was started this frame because of a redraw request.
    redraw_request: bool,
    /// Tracks if the event loop was started this frame because of a `ControlFlow::WaitUntil`
    /// timeout.
    timeout_reached: bool,
    wait_cancelled: bool,
    last_update: Instant,
    mode: FlowMode,
    cursor_moved: bool,
}

impl Default for WinitPersistentState {
    fn default() -> Self {
        Self {
            active: false,
            redraw_request: false,
            timeout_reached: false,
            wait_cancelled: false,
            last_update: Instant::now(),
            mode: FlowMode::Wait,
            cursor_moved: false,
        }
    }
}
