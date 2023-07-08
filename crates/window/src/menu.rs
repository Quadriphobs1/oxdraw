// TODO: Even though this should be forbidden by default. Menu is currently not implemented for linux platform.
// Disable a bunch of the warning until Linux platform menu is fixed
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(clippy::unnecessary_wraps)]
use std::{collections::HashMap, path::Path, process::Command, str};

use anyhow::{bail, Context};
use glam::Vec2;
#[cfg(target_os = "windows")]
use muda::ContextMenu;
use muda::{
    accelerator::{Accelerator, Code, Modifiers},
    icon::Icon,
    CheckMenuItem, Menu, MenuEvent, MenuEventReceiver, MenuItem, PredefinedMenuItem, Submenu,
};
#[cfg(target_os = "macos")]
use muda::{AboutMetadata, ContextMenu};
#[cfg(target_os = "macos")]
use winit::platform::macos::{EventLoopBuilderExtMacOS, WindowExtMacOS};
#[cfg(target_os = "windows")]
use winit::platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows};
use winit::{event_loop::EventLoopBuilder, window::Window};

use crate::error::WindowsError;

#[cfg(target_os = "macos")]
pub const CMD_OR_CTRL: Modifiers = Modifiers::META;
#[cfg(not(target_os = "macos"))]
pub const CMD_OR_CTRL: Modifiers = Modifiers::CONTROL;

#[derive(Eq, Hash, PartialEq)]
pub enum SubMenuKind {
    #[cfg(target_os = "macos")]
    App,
    Edit,
    File,
    Help,
    Window,
    View,
    Custom,
}

pub struct MenuManager {
    menu_bar: Menu,
    menus: HashMap<SubMenuKind, Submenu>,
    channel: &'static MenuEventReceiver,
    installed_global: bool,
}
/// Constructor member
impl MenuManager {
    pub fn new(event_loop_builder: &mut EventLoopBuilder<()>) -> MenuManager {
        let menu_bar = Menu::new();
        #[cfg(target_os = "linux")]
        {
            // menu.init_for_gtk_window(&gtk_window);
        }
        #[cfg(target_os = "windows")]
        {
            let menu_bar_c = menu_bar.clone();
            event_loop_builder.with_msg_hook(move |msg| {
                use windows_sys::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, MSG};
                #[allow(unsafe_code)]
                unsafe {
                    let msg = msg as *const MSG;
                    let translated = TranslateAcceleratorW((*msg).hwnd, menu_bar_c.haccel(), msg);
                    translated == 1
                }
            });
        }
        #[cfg(target_os = "macos")]
        event_loop_builder.with_default_menu(false);

        let menu_channel = MenuEvent::receiver();

        MenuManager {
            menu_bar,
            menus: HashMap::new(),
            channel: menu_channel,
            installed_global: false,
        }
    }
}

/// Mutable member
impl MenuManager {
    /// Install the menu to the window
    pub fn install(&mut self, window: &Window) -> anyhow::Result<()> {
        // TODO(Quadri): Fix for linux/unix/x11/wayland platform, [`muda`] currently supports [`gtk`] only linux
        #[cfg(target_os = "linux")]
        {
            // self.menu_bar.init_for_gtk_window(_window);
        }

        #[cfg(target_os = "windows")]
        {
            self.menu_bar.init_for_hwnd(window.hwnd());
        }

        #[cfg(target_os = "macos")]
        {
            self.menu_bar.init_for_nsapp();
            if !self.installed_global {
                self.sub_menu(&SubMenuKind::App)?
                    .set_windows_menu_for_nsapp();
                self.sub_menu(&SubMenuKind::Help)?.set_help_menu_for_nsapp();
            }
        }

        self.installed_global = true;
        Ok(())
    }

    pub fn setup(&mut self) -> anyhow::Result<()> {
        // Add individual menu in order they appear in the menu bar
        #[cfg(target_os = "macos")]
        {
            self.add_app_menu()?;
        }
        self.add_file_menu();
        self.add_edit_menu();
        self.add_view_menu();
        self.add_window_menu();
        self.add_help_menu();
        Ok(())
    }

    pub fn listen(&self) {
        if let Ok(event) = self.channel.try_recv() {
            println!("listening to menu event {event:?}");
        }
    }

    pub fn show_context_menu(window: &Window, menu: &SubMenuKind, position: Vec2) {
        if menu != &SubMenuKind::Custom {
            return;
        }
        let custom = Submenu::new("Custom", true);

        custom.append_items(&[
            &MenuItem::new(
                "Custom 1",
                true,
                Some(Accelerator::new(Some(Modifiers::ALT), Code::KeyC)),
            ),
            &MenuItem::new(
                "Custom 2",
                true,
                Some(Accelerator::new(Some(Modifiers::ALT), Code::KeyC)),
            ),
            &MenuItem::new(
                "Custom 3",
                true,
                Some(Accelerator::new(Some(Modifiers::ALT), Code::KeyC)),
            ),
        ]);
        // TODO(Quadri): Fix for linux/unix/x11/wayland platform, [`muda`] currently supports [`gtk`] only linux
        #[cfg(target_os = "linux")]
        {
            // custom.show_context_menu_for_gtk_window(
            //     window.ns_view().cast(),
            //     position.x as f64,
            //     position.y as f64,
            // );
        }
        #[cfg(target_os = "windows")]
        {
            custom.show_context_menu_for_hwnd(window.hwnd(), position.x as f64, position.y as f64);
        }
        #[cfg(target_os = "macos")]
        {
            custom.show_context_menu_for_nsview(
                window.ns_view().cast(),
                position.x as f64,
                position.y as f64,
            );
        }
    }
}

/// Reference member
impl MenuManager {
    fn sub_menu(&self, kind: &SubMenuKind) -> anyhow::Result<&Submenu> {
        match self.menus.get(kind) {
            Some(menu) => Ok(menu),
            None => bail!("Error getting menu"),
        }
    }

    #[cfg(target_os = "macos")]
    fn add_app_menu(&mut self) -> anyhow::Result<()> {
        let mut workspace_dir = get_workspace_dir()?;
        let icon_path = "/assets/icon.png";
        workspace_dir.push_str(icon_path);
        let app_name = env!("APP_NAME");
        let version = env!("CARGO_PKG_VERSION");
        let desc = env!("APP_DESC");
        let author = env!("APP_AUTHOR");
        let license = env!("CARGO_PKG_LICENSE");
        let website = env!("CARGO_PKG_HOMEPAGE");
        let icon: Icon = load_app_icon(std::path::Path::new(&workspace_dir))?;

        let settings = MenuItem::new(
            "Settings",
            true,
            Some(Accelerator::new(Some(Modifiers::META), Code::Comma)),
        );

        let app = Submenu::with_items(
            app_name,
            true,
            &[
                &PredefinedMenuItem::about(
                    Some("About Oxdraw"),
                    Some(AboutMetadata {
                        name: Some(app_name.to_owned()),
                        version: Some(version.to_owned()),
                        comments: Some(desc.to_owned()),
                        authors: Some(vec![author.to_owned()]),
                        license: Some(license.to_owned()),
                        website: Some(website.to_owned()),
                        website_label: Some("Github".to_owned()),
                        icon: Some(icon),
                        copyright: Some(
                            "Copyright (c) Quadri Adekunle 2023. All rights reserved.".to_owned(),
                        ),
                        ..Default::default()
                    }),
                ),
                &PredefinedMenuItem::separator(),
                &settings,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::services(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ],
        );
        self.menu_bar.append(&app);
        self.menus.insert(SubMenuKind::App, app);

        Ok(())
    }

    fn add_file_menu(&mut self) {
        // CmdOrCtrl+N
        let new = MenuItem::new(
            "New",
            true,
            Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyN)),
        );
        // CmdOrCtrl+Shift+N
        let new_window = MenuItem::new(
            "New Window",
            true,
            Some(Accelerator::new(
                Some(CMD_OR_CTRL | Modifiers::SHIFT),
                Code::KeyN,
            )),
        );
        // CmdOrCtrl+O
        let open = MenuItem::new(
            "Open",
            true,
            Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyO)),
        );
        // CmdOrCtrl+Shift+O
        let open_recent = MenuItem::new(
            "Open Recent",
            true,
            Some(Accelerator::new(
                Some(CMD_OR_CTRL | Modifiers::SHIFT),
                Code::KeyO,
            )),
        );
        // CmdOrCtrl+O
        let save = MenuItem::new(
            "Save",
            true,
            Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyS)),
        );
        // CmdOrCtrl+Shift+S
        let save_as = MenuItem::new(
            "Save",
            true,
            Some(Accelerator::new(
                Some(CMD_OR_CTRL | Modifiers::SHIFT),
                Code::KeyS,
            )),
        );
        let file = Submenu::with_items(
            "File",
            true,
            &[
                &new,
                &new_window,
                &PredefinedMenuItem::separator(),
                &open,
                &open_recent,
                &PredefinedMenuItem::separator(),
                &save,
                &save_as,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::close_window(Some("Close")),
            ],
        );
        self.menu_bar.append(&file);
        self.menus.insert(SubMenuKind::File, file);
    }

    fn add_edit_menu(&mut self) {
        // CmdOrCtrl+Z
        let undo = MenuItem::new(
            "Undo",
            true,
            Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyZ)),
        );

        // CmdOrCtrl+Shift+Z
        let redo = MenuItem::new(
            "Redo",
            true,
            Some(Accelerator::new(
                Some(CMD_OR_CTRL | Modifiers::SHIFT),
                Code::KeyZ,
            )),
        );

        let delete = MenuItem::new("Delete", true, None);

        // CmdOrCtrl+Shift+A
        let select_all = MenuItem::new(
            "Select All",
            true,
            Some(Accelerator::new(
                Some(CMD_OR_CTRL | Modifiers::SHIFT),
                Code::KeyA,
            )),
        );

        let copy = PredefinedMenuItem::copy(None);
        let cut = PredefinedMenuItem::cut(None);
        let paste = PredefinedMenuItem::paste(None);

        let edit = Submenu::with_items(
            "Edit",
            true,
            &[
                &undo,
                &redo,
                &PredefinedMenuItem::separator(),
                &cut,
                &copy,
                &paste,
                &delete,
                &select_all,
            ],
        );
        self.menu_bar.append(&edit);
        self.menus.insert(SubMenuKind::Edit, edit);
    }

    fn add_view_menu(&mut self) {
        // CmdOrCtrl++
        let zoom_in = MenuItem::new(
            "Zoom In",
            true,
            Some(Accelerator::new(Some(CMD_OR_CTRL), Code::Equal)),
        );
        // CmdOrCtrl+-
        let zoom_out = MenuItem::new(
            "Zoom Out",
            true,
            Some(Accelerator::new(Some(CMD_OR_CTRL), Code::Minus)),
        );
        let zoom_reset = MenuItem::new("Reset Zoom", true, None);
        let view = Submenu::with_items(
            "View",
            true,
            &[
                &zoom_in,
                &zoom_out,
                &zoom_reset,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::fullscreen(None),
            ],
        );
        self.menu_bar.append(&view);
        self.menus.insert(SubMenuKind::View, view);
    }

    fn add_window_menu(&mut self) {
        // TODO(Quadri): Show list of available windows
        let current_window = CheckMenuItem::new("Oxdraw - current window", false, true, None);
        let window = Submenu::with_items(
            "Window",
            true,
            &[
                &PredefinedMenuItem::minimize(None),
                &PredefinedMenuItem::maximize(None),
                &PredefinedMenuItem::close_window(Some("Close")),
                &PredefinedMenuItem::fullscreen(None),
                &PredefinedMenuItem::separator(),
                &current_window,
            ],
        );
        self.menu_bar.append(&window);
        self.menus.insert(SubMenuKind::Window, window);
    }

    fn add_help_menu(&mut self) {
        let feedback = MenuItem::new("Send feedback", true, None);
        let help = MenuItem::new("Oxdraw help", true, None);
        let shortcuts = MenuItem::new("Keyboard shortcuts", true, None);
        let github = MenuItem::new("Github", true, None);
        let about = MenuItem::new("About Oxdraw", true, None);

        let help = Submenu::with_items(
            "Help",
            true,
            &[
                &feedback,
                &PredefinedMenuItem::separator(),
                &help,
                &shortcuts,
                &PredefinedMenuItem::separator(),
                &github,
                &about,
            ],
        );
        self.menu_bar.append(&help);
        self.menus.insert(SubMenuKind::Help, help);
    }
}

fn load_app_icon(path: &Path) -> anyhow::Result<Icon> {
    let (rgba, width, height) = {
        let image = image::open(path)?.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(rgba, width, height)?;

    Ok(icon)
}

/// Get the workspace directory. Because the app is in a workspace and several packages,
/// the env variable `CARGO_MANIFEST_DIR` no longer satisfy the root directory.
/// This is a hack method to get the workspace directory from any package deep in the workspace.
fn get_workspace_dir() -> anyhow::Result<String> {
    let command = env!("CARGO");
    let args = ["locate-project", "--workspace", "--message-format=plain"];
    let output = Command::new(command).args(args).output()?;

    let cmd_line: String = format!("{command} {}", args.join(" "));
    let stdout = extract_stdout(&cmd_line, &output)?;

    let cargo_path = Path::new(stdout.trim());
    let parent_path = cargo_path
        .parent()
        .with_context(|| "Unable to get the parent")?;
    let workspace = parent_path.display().to_string();
    Ok(workspace)
}

fn extract_stdout<'a>(
    cmd_line: &'_ str,
    output: &'a std::process::Output,
) -> anyhow::Result<&'a str> {
    if !output.status.success() {
        bail!(WindowsError::ProcessOutputError(
            cmd_line.to_owned(),
            output.status.to_string(),
            output.stderr.escape_ascii().to_string()
        ));
    }

    let stdout = str::from_utf8(&output.stdout)?;

    Ok(stdout)
}
