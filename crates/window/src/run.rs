use std::{
    rc::Rc,
    thread,
    time::{Duration, Instant},
};

use glam::Vec2;
use log::{error, info};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, Event, StartCause, Touch, WindowEvent as WNWindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
};

use crate::{
    event::{MouseButton, MouseEvent, MouseTouchEvent},
    FlowMode, Windows,
};

fn run<F>(event_loop: EventLoop<()>, event_handler: F) -> !
where
    F: 'static + FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
{
    event_loop.run(event_handler)
}

/// Winit event loop runner. Main window level actions are controlled through this function.
pub fn winit_runner(mut windows: Windows) {
    info!("Entering winit event loop");
    let mut context = windows.ctx;
    let mut winit_state = windows.state;

    // Every created window is initially set to invisible by default. Set the main application window to visible,
    // every other window would be set to visible on request.
    let window = match windows.wm.main_window() {
        Ok(window) => window,
        Err(err) => {
            error!("Widow initialization error: {}", err);
            return;
        }
    };
    if let Err(err) = windows.menu.install(window) {
        error!("Unable to setup app menu: {}", err);
        return;
    };
    window.set_visible(true);
    // Setup the 2D renderer for the primary window.
    let PhysicalSize { width, height } = window.inner_size();
    let id = u64::from(window.id());
    if let Err(err) = windows.renderer.setup(id, width, height, &window) {
        error!("Unable to setup a renderer for window: {}", err);
        return;
    }

    // We `take` this so that we have ownership over it. By reaching this point,
    // all resources depending on the event_loop should have released their ownership
    // Note: This will panic if the value is still borrowed.
    let event_loop = Rc::try_unwrap(windows.event_loop)
        .unwrap_or_else(|_| panic!("There is still a reference to event_loop, failed to unwrap"))
        .into_inner();

    let event_handler = move |event: Event<'_, ()>,
                              _target: &EventLoopWindowTarget<()>,
                              control_flow: &mut ControlFlow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        // *control_flow = ControlFlow::Poll;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        control_flow.set_wait();

        match event {
            Event::NewEvents(start_cause) => match start_cause {
                StartCause::Init => {
                    info!("Winit app has started");
                    winit_state.wait_cancelled = false;
                }
                StartCause::WaitCancelled { .. } => {
                    winit_state.wait_cancelled = winit_state.mode == FlowMode::WaitUntil;
                }
                // Check if either the `WaitUntil` timeout was triggered by winit, or that same
                // amount of time has elapsed since the last app update. This manual check is needed
                // because we don't know if the criteria for an app update were met until the end of
                // the frame.
                StartCause::ResumeTimeReached { .. } => {
                    winit_state.timeout_reached = true;
                    winit_state.wait_cancelled = false;
                }
                StartCause::Poll => {
                    winit_state.wait_cancelled = false;
                }
            },
            Event::WindowEvent { window_id, event } => {
                if windows.wm.window(&window_id).is_none() {
                    error!(
                        "Skipped event {:?} for unknown winit Window Id {:?}",
                        event, window_id
                    );
                    // Do not continue process the event
                    return;
                }
                match event {
                    WNWindowEvent::Destroyed => {
                        info!("Closing window {:?}", window_id);
                    }
                    WNWindowEvent::CloseRequested => {
                        info!("Closing window {:?}", window_id);
                        if let Err(err) =
                            context.remove(&mut windows.wm, &mut windows.renderer, &window_id)
                        {
                            error!("Unable to clear resources for window: {}", err);
                        };
                        if windows.wm.is_empty() {
                            info!("Closing all winit window");
                            control_flow.set_exit();
                        }
                    }
                    WNWindowEvent::Resized(size) => {
                        info!("Resizing the window");
                        if let Err(err) =
                            context.resize(&mut windows.wm, &mut windows.renderer, &window_id, size)
                        {
                            error!("Unable to resize the window: {}", err);
                        }
                    }
                    WNWindowEvent::ScaleFactorChanged {
                        scale_factor,
                        new_inner_size,
                    } => {
                        info!("Scaling the window");

                        if let Err(err) = context.scale_factor(
                            &mut windows.wm,
                            &mut windows.renderer,
                            &window_id,
                            scale_factor,
                            new_inner_size,
                        ) {
                            error!("Unable to scale the window: {}", err);
                        }
                    }
                    WNWindowEvent::Focused(focus) => {
                        if focus {
                            info!("Changing focussed window to {window_id:?}");
                        } else {
                            info!("Changing focussed window from {window_id:?}");
                        }
                        if let Err(err) = context.focused(&mut windows.wm, &window_id, focus) {
                            error!("Unable to focus window: {}", err);
                        }
                    }
                    WNWindowEvent::Touch(Touch {
                        phase, location, ..
                    }) => {
                        if let Err(err) =
                            context.touch(&mut windows.wm, &window_id, phase, location)
                        {
                            error!("Unable to process touch event for window: {}", err);
                        }
                    }
                    WNWindowEvent::KeyboardInput { event, .. } => {
                        if let Err(err) = context.key_input(&mut windows.wm, &window_id, &event) {
                            error!("Unable to process key received for window: {}", err);
                        }
                    }
                    WNWindowEvent::ModifiersChanged(modifier) => {
                        if let Err(err) =
                            context.key_modifier(&mut windows.wm, &window_id, &modifier.state())
                        {
                            error!("Unable to process cursor move for window: {}", err);
                        }
                    }
                    // TODO(Quadri): Implement IME event for key board input
                    // WNWindowEvent::Ime(_) => {

                    // }
                    WNWindowEvent::CursorMoved { position, .. } => {
                        // To avoid calling the hover system multiple times in one frame when multiple cursor moved
                        // events are received, instead we set a flag here and emit the MouseMove event during MainEventsCleared.
                        if !winit_state.cursor_moved {
                            winit_state.cursor_moved = true;
                        }
                        if let Err(err) = context.cursor_move(&mut windows.wm, &window_id, position)
                        {
                            error!("Unable to process cursor move for window: {}", err);
                        }
                    }
                    WNWindowEvent::MouseWheel { delta, .. } => {
                        if let Err(err) = context.mouse_wheel(&mut windows.wm, &window_id, delta) {
                            error!("Unable to process mouse wheel for window: {}", err);
                        }
                    }
                    WNWindowEvent::MouseInput { state, button, .. } => {
                        let button = match button {
                            winit::event::MouseButton::Left => MouseButton::Left,
                            winit::event::MouseButton::Right => MouseButton::Right,
                            winit::event::MouseButton::Middle => MouseButton::Middle,
                            winit::event::MouseButton::Back => MouseButton::Back,
                            winit::event::MouseButton::Forward => MouseButton::Forward,
                            winit::event::MouseButton::Other(val) => MouseButton::Other(val),
                        };

                        let event = match state {
                            winit::event::ElementState::Pressed => MouseTouchEvent::Down(button),
                            winit::event::ElementState::Released => MouseTouchEvent::Up(button),
                        };

                        if let Err(err) = context.mouse_input(&mut windows.wm, &window_id, &event) {
                            error!("Unable to process mouse input for window: {}", err);
                        }
                    }
                    _ => {}
                }
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (x, y) },
                ..
            } => {
                if let Err(err) = context.mouse_move(&mut windows.wm, Vec2::new(x as f32, y as f32))
                {
                    error!("Unable process mouse motion: {}", err);
                }
            }
            Event::Suspended => {
                info!("Winit app suspended");
                winit_state.active = false;
            }
            Event::Resumed => {
                info!("Winit app has resumed");
                winit_state.active = true;
            }
            Event::MainEventsCleared => {
                // Application update code.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw, in
                // applications which do not always need to. Applications that redraw continuously
                // can just render here instead.

                if !winit_state.active {
                    return;
                }
                if winit_state.cursor_moved {
                    context.process_mouse_event(&MouseEvent::Move(context.mouse_pos));
                    winit_state.cursor_moved = false;
                }

                if !winit_state.redraw_request && !winit_state.timeout_reached {
                    return;
                }

                winit_state.last_update = Instant::now();
                if let Err(err) = context.update(&windows.wm, &mut windows.renderer) {
                    error!("error : {}", err);
                };
            }
            Event::RedrawRequested(window_id) => {
                info!("Processing redraw request");
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in MainEventsCleared, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.
                if let Err(err) = context.render(&mut windows.wm, &mut windows.renderer, &window_id)
                {
                    error!("Unable render content for window {:?} : {}", window_id, err);
                    // We exit the application while logging the error if unable to render content into the window.
                    control_flow.set_exit();
                };
            }
            Event::RedrawEventsCleared => {
                let now = Instant::now();

                match winit_state.mode {
                    FlowMode::Wait => control_flow.set_wait(),
                    FlowMode::WaitUntil => {
                        if !winit_state.wait_cancelled {
                            control_flow.set_wait_until(now + Duration::from_millis(100));
                        }
                    }
                    FlowMode::Poll => {
                        thread::sleep(Duration::from_millis(100));
                        control_flow.set_poll();
                        winit_state.redraw_request = true;
                    }
                };
            }
            _ => (),
        }

        windows.menu.listen();
    };

    run(event_loop, event_handler)
}
