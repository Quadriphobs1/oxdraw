use anyhow::bail;
use glam::Vec2;
use log::info;
use renderer::Renderer;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{KeyEvent as WKeyEvent, MouseScrollDelta, TouchPhase},
    keyboard::ModifiersState,
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::WindowId,
};

use crate::{
    error::WindowsError,
    event::{KeyEvent, KeyboardModifiers, MouseEvent, MouseTouchEvent, TouchInputEvent},
    key_code::key_event_to_code,
    menu::{MenuManager, SubMenuKind},
    window::WindowManager,
};

#[derive(Default)]
pub struct Context {
    pub mouse_pos: Vec2,
    has_rendered: bool,
    window_size: PhysicalSize<u32>,
    /// Lock the cursor in position. Useful for dragging knobs.
    grab_cursor: bool,

    /// Value of grab_cursor before processing event.
    prev_grab_cursor: bool,

    /// Keyboard modifiers state.
    pub key_mods: KeyboardModifiers,
}

impl Context {
    /// Process a UI event from window to the renderer.
    pub fn process_touch(&mut self, event: &TouchInputEvent) {
        println!("Process mouse touch event {event:?}");
        self.has_rendered = true;
    }

    pub fn process_mouse_event(&mut self, event: &MouseEvent) {
        println!("Process mouse move event {event:?}",);

        // Check if the mouse move event is within the bounds of any active window.

        self.has_rendered = true;
    }

    pub fn process_key_event(&mut self, event: &KeyEvent) {
        println!("Process mouse move event {event:?}",);

        self.has_rendered = true;
    }

    // Update the UI of the window
    pub fn render(
        &mut self,
        wm: &mut WindowManager,
        renderer: &mut Renderer,
        window_id: &WindowId,
    ) -> anyhow::Result<()> {
        let Some(window) = wm.window(window_id) else {
            bail!(WindowsError::WindowStore(*window_id));
         };

        let PhysicalSize { width, height } = window.inner_size();
        let id = u64::from(*window_id);
        renderer.render(id, width, height, window.scale_factor())?;
        self.has_rendered = true;
        Ok(())
    }

    /// Resize the window and renderer, redraw if necessary
    pub fn resize(
        &mut self,
        wm: &mut WindowManager,
        renderer: &mut Renderer,
        window_id: &WindowId,
        size: PhysicalSize<u32>,
    ) -> anyhow::Result<()> {
        let Some(window) = wm.window(window_id) else {
            bail!(WindowsError::WindowStore(*window_id));
        };
        let id = u64::from(window.id());
        renderer.update(id, size.width, size.height)?;
        window.request_redraw();
        self.has_rendered = true;

        Ok(())
    }

    /// Change the scale factor of the window and renderer, redraw if necessary
    pub fn scale_factor(
        &mut self,
        wm: &mut WindowManager,
        renderer: &mut Renderer,
        window_id: &WindowId,
        scale_factor: f64,
        size: &mut PhysicalSize<u32>,
    ) -> anyhow::Result<()> {
        // the OS suggested size, We have already told the OS about our resize constraints,
        // so the size should take those into account
        let Some(window) = wm.window(window_id) else {
            bail!(WindowsError::WindowStore(*window_id));
        };

        let window_size = window.inner_size();

        let new_width = size.width as f64 / scale_factor;
        let new_height = size.height as f64 / scale_factor;
        if approx::relative_eq!(window_size.width as f64, new_width)
            || approx::relative_eq!(window_size.height as f64, new_height)
        {
            return Ok(());
        }

        let id = u64::from(window.id());
        renderer.update(id, new_width as u32, new_height as u32)?;
        window.request_redraw();
        self.has_rendered = true;
        Ok(())
    }

    /// Update the focus state of the window
    pub fn focused(
        &mut self,
        wm: &mut WindowManager,
        window_id: &WindowId,
        focus: bool,
    ) -> anyhow::Result<()> {
        // TODO(Quadri): If the window is minimized or not visible, this method should make it visible
        let Some(window) = wm.window(window_id) else {
            bail!(WindowsError::Focus(*window_id));
        };
        if !wm.is_current_window(window_id) && !window.has_focus() && focus {
            window.focus_window();
            wm.focus_window(*window_id);
        }
        self.has_rendered = true;

        Ok(())
    }

    /// Handle user key input
    pub fn key_input(
        &mut self,
        wm: &mut WindowManager,
        window_id: &WindowId,
        event: &WKeyEvent,
    ) -> anyhow::Result<()> {
        if wm.window(window_id).is_none() {
            bail!(WindowsError::WindowStore(*window_id));
        }

        let code = key_event_to_code(event.key_without_modifiers(), &self.key_mods);

        let event = match event.state {
            winit::event::ElementState::Pressed => KeyEvent::Down(code, event.repeat),
            winit::event::ElementState::Released => KeyEvent::Up(code),
        };
        self.process_key_event(&event);

        Ok(())
    }

    /// Handles key modifier such as cmd, alt, etc
    pub fn key_modifier(
        &mut self,
        wm: &mut WindowManager,
        window_id: &WindowId,
        state: &ModifiersState,
    ) -> anyhow::Result<()> {
        if wm.window(window_id).is_none() {
            bail!(WindowsError::WindowStore(*window_id));
        }
        self.key_mods = KeyboardModifiers {
            shift: state.shift_key(),
            control: state.control_key(),
            alt: state.alt_key(),
            command: state.super_key(),
        };
        Ok(())
    }

    /// User have touched into the window
    /// - **macOS:** Unsupported.
    pub fn touch(
        &mut self,
        wm: &mut WindowManager,
        window_id: &WindowId,
        phase: TouchPhase,
        location: PhysicalPosition<f64>,
    ) -> anyhow::Result<()> {
        let Some(window) = wm.window(window_id) else {
            bail!(WindowsError::WindowStore(*window_id));
        };

        let scale = window.scale_factor() as f32;
        let position = Vec2::new(location.x as f32 / scale, location.y as f32 / scale);
        // TODO(Quadri): Delta should compare to prev position
        let delta = position;

        let event = match phase {
            TouchPhase::Started => TouchInputEvent::Begin { id: 0, position },
            TouchPhase::Moved => TouchInputEvent::Move {
                id: 0,
                position,
                delta,
            },
            TouchPhase::Ended | TouchPhase::Cancelled => TouchInputEvent::End { id: 0, position },
        };

        self.process_touch(&event);
        Ok(())
    }

    /// Handle when the user cursor has moved onto the window
    pub fn cursor_move(
        &mut self,
        wm: &mut WindowManager,
        window_id: &WindowId,
        position: PhysicalPosition<f64>,
    ) -> anyhow::Result<()> {
        let Some(window) = wm.window(window_id) else {
            bail!(WindowsError::WindowStore(*window_id));
        };

        let scale = window.scale_factor() as f32;
        let position = Vec2::new(position.x as f32 / scale, position.y as f32 / scale);

        self.mouse_pos = position;

        Ok(())
    }

    pub fn mouse_wheel(
        &mut self,
        wm: &mut WindowManager,
        window_id: &WindowId,
        delta: MouseScrollDelta,
    ) -> anyhow::Result<()> {
        // TODO(Quadri): Check if the current window supports scrolling with its content.
        if wm.window(window_id).is_none() {
            bail!(WindowsError::WindowStore(*window_id));
        }

        let event = match delta {
            MouseScrollDelta::LineDelta(x, y) => MouseEvent::Scroll(Vec2::new(x, y)),
            MouseScrollDelta::PixelDelta(pos) => {
                MouseEvent::Scroll(Vec2::new(pos.x as f32, pos.y as f32))
            }
        };

        self.process_mouse_event(&event);

        Ok(())
    }

    pub fn mouse_input(
        &mut self,
        wm: &mut WindowManager,
        window_id: &WindowId,
        event: &MouseTouchEvent,
    ) -> anyhow::Result<()> {
        let Some(window) = wm.window(window_id) else {
            bail!(WindowsError::WindowStore(*window_id));
        };
        let touch = match event {
            MouseTouchEvent::Down(_) => TouchInputEvent::Begin {
                id: 0,
                position: self.mouse_pos,
            },
            MouseTouchEvent::Up(_) => TouchInputEvent::End {
                id: 0,
                position: self.mouse_pos,
            },
        };

        MenuManager::show_context_menu(window, &SubMenuKind::Custom, self.mouse_pos);

        self.process_touch(&touch);
        Ok(())
    }

    pub fn mouse_move(&mut self, wm: &mut WindowManager, delta: Vec2) -> anyhow::Result<()> {
        let window = wm.current_window()?;
        if self.mouse_pos == delta {
            return Ok(());
        }
        let event = TouchInputEvent::Move {
            id: 0,
            position: self.mouse_pos,
            delta,
        };
        self.process_touch(&event);

        // TODO(Quadri): Grab cursor should set when mouse down happens on any element/view that accepts mouse drag
        if self.grab_cursor && !self.prev_grab_cursor {
            info!("grabbing cursor");
            window
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Confined))
                .unwrap();
            window.set_cursor_visible(false);
        }

        if !self.grab_cursor && self.prev_grab_cursor {
            info!("releasing cursor");
            window
                .set_cursor_grab(winit::window::CursorGrabMode::None)
                .unwrap();
            window.set_cursor_visible(true);
        }

        self.prev_grab_cursor = self.grab_cursor;
        Ok(())
    }

    /// Queue a redraw request for focused window
    /// Call this after the event queue is cleared.
    pub fn update(&mut self, wm: &WindowManager, renderer: &mut Renderer) -> anyhow::Result<()> {
        let window = wm.current_window()?;

        let window_size = window.inner_size();
        let scale = window.scale_factor() as f32;
        let width = window_size.width as f32 / scale;
        let height = window_size.height as f32 / scale;

        // If the window size has changed, force a re-layout.
        if window_size != self.window_size {
            self.window_size = window_size;
        }
        let id = u64::from(window.id());
        renderer.update(id, width as u32, height as u32)?;

        // TODO(Quadri): perform re-layout of views

        // Update the current window
        window.request_redraw();
        self.has_rendered = true;

        Ok(())
    }

    /// Remove every registered resources/process/events for a window
    pub fn remove(
        &mut self,
        wm: &mut WindowManager,
        renderer: &mut Renderer,
        window_id: &WindowId,
    ) -> anyhow::Result<()> {
        // TODO(Quadri): Handle switching focus between closed window to main window
        // This drops the window, causing it to close.
        wm.remove(window_id);
        let id = u64::from(*window_id);
        renderer.remove(id)?;

        self.has_rendered = false;
        self.window_size = PhysicalSize::default();
        Ok(())
    }
}
