/*!

This module contains all Nexus-specific UI rendering logic and components for the Guild Wars 2 executable runner addon.

## Components

- Main window rendering
- Executable list and controls
- Add executable dialog
- Control buttons (Stop All, Running Count)

*/

use crate::addon::manager::{open_file_dialog, ExeManager, Executable, EXE_MANAGER};
use nexus::{
    gui::register_render,
    imgui::{Ui, Window},
    render,
};
use std::sync::atomic::{AtomicBool, Ordering};

/// Global state for tracking if the main window is open
pub static IS_WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

/// Registers the main window rendering callback with nexus
pub fn setup_main_window_rendering() {
    let main_window = render!(|ui| {
        render_main_window(ui);
    });
    register_render(nexus::gui::RenderType::Render, main_window).revert_on_unload();
}

/// Renders the main window
pub fn render_main_window(ui: &Ui) {
    let mut is_open = IS_WINDOW_OPEN.load(Ordering::Relaxed);
    if is_open {
        Window::new("Gw2 Executable Runner")
            .opened(&mut is_open)
            .size([500.0, 400.0], nexus::imgui::Condition::FirstUseEver)
            .collapsible(false)
            .build(ui, || {
                render_window_content(ui);
            });
        IS_WINDOW_OPEN.store(is_open, Ordering::Relaxed);
    }
}

/// Renders the content inside the main window
fn render_window_content(ui: &Ui) {
    if let Some(exe_manager_arc) = EXE_MANAGER.get() {
        let mut exe_manager = exe_manager_arc.lock().unwrap();

        // Cleanup finished processes
        exe_manager.cleanup_finished_processes();

        render_header(ui);
        render_add_executable_section(ui, &mut exe_manager);
        render_executable_list(ui, &mut exe_manager);
        render_control_buttons(ui, &exe_manager);
    }
}

/// Renders the window header
fn render_header(ui: &Ui) {
    ui.text("Gw2 Executable Runner");
    ui.separator();
    ui.text_wrapped("To start an executable, please select an executable file below.");
    ui.text_wrapped("Then, launch executable with the 'Launch' button.");
    ui.text_wrapped("You can make it launch automatically on startup by checking on the checkbox next to the executable.");
    ui.separator();
}

/// Renders the section for adding new executables
fn render_add_executable_section(ui: &Ui, exe_manager: &mut ExeManager) {
    ui.text("Add New Executable:");

    if ui.button("Browse for Executable...") {
        if let Some(selected_path) = open_file_dialog() {
            if let Err(e) = exe_manager.add_exe(selected_path) {
                log::error!("Failed to add executable: {e}");
            }
        }
    }

    ui.same_line();
    ui.text("Click 'Browse' to select an executable file");
    ui.separator();
}

/// Renders the list of executables with their controls
fn render_executable_list(ui: &Ui, exe_manager: &mut ExeManager) {
    ui.text("Executable List:");

    // Track actions to perform after the loop
    let mut to_remove = None;
    let mut to_stop = None;
    let mut to_launch = None;

    // Clone the paths to avoid borrowing issues
    let exe_paths = exe_manager.exe_paths().clone();

    if exe_paths.is_empty() {
        ui.text_colored([0.6, 0.6, 0.6, 1.0], "No executable configured");
    }

    for (i, exe_path) in exe_paths.iter().enumerate() {
        let is_running = exe_manager.is_running(exe_path);

        let _id = ui.push_id(i as i32);

        render_executable_item(
            exe_manager,
            ui,
            exe_path,
            is_running,
            &mut to_launch,
            &mut to_stop,
            &mut to_remove,
            i,
        );
    }

    // Handle actions after the loop to avoid borrowing conflicts
    handle_executable_actions(exe_manager, to_stop, to_launch, to_remove);
}

/// Renders a single executable item in the list
fn render_executable_item(
    exe_manager: &mut ExeManager,
    ui: &Ui,
    exe_path: &str,
    is_running: bool,
    to_launch: &mut Option<String>,
    to_stop: &mut Option<String>,
    to_remove: &mut Option<usize>,
    index: usize,
) {

    let exe: &Executable = exe_manager.executables().get(index).unwrap();

    // Status indicator
    if is_running {
        ui.text_colored([0.0, 1.0, 0.0, 1.0], "Running");
    } else {
        ui.text_colored([0.5, 0.5, 0.5, 1.0], "Not running");
    }
    ui.same_line();

    // Executable path (truncated if too long)
    let display_path = if exe_path.len() > 50 {
        format!("...{}", &exe_path[exe_path.len() - 47..])
    } else {
        exe_path.to_string()
    };
    ui.text(&display_path);

    ui.same_line();

    // Auto-launch checkbox
    if ui.checkbox("Launch on startup", exe_manager.launch_on_startup(&exe)) {
        if let Err(e) = exe_manager.save_settings() {
            log::error!("Failed to save settings: {e}");
        }
    }

    // Launch/Stop button
    if is_running {
        if ui.button("Stop") {
            *to_stop = Some(exe_path.to_string());
        }
    } else if ui.button("Launch") {
        *to_launch = Some(exe_path.to_string());
    }

    ui.same_line();

    // Remove button
    if ui.button("Remove") {
        *to_remove = Some(index);
    }
}

/// Handles the actions collected during executable list rendering
fn handle_executable_actions(
    exe_manager: &mut ExeManager,
    to_stop: Option<String>,
    to_launch: Option<String>,
    to_remove: Option<usize>,
) {
    if let Some(path) = to_stop {
        if let Err(e) = exe_manager.stop_exe(&path) {
            log::error!("Failed to stop executable: {e}");
        }
    }

    if let Some(path) = to_launch {
        if let Err(e) = exe_manager.launch_exe(&path) {
            log::error!("Failed to launch executable: {e}");
        }
    }

    if let Some(index) = to_remove {
        if let Err(e) = exe_manager.remove_exe(index) {
            log::error!("Failed to remove executable: {e}");
        }
    }
}

/// Renders the control buttons section
fn render_control_buttons(ui: &Ui, exe_manager: &ExeManager) {
    ui.separator();

    if ui.button("Stop All") {
        if let Some(exe_manager_arc) = EXE_MANAGER.get() {
            let mut exe_manager = exe_manager_arc.lock().unwrap();
            if let Err(e) = exe_manager.stop_all() {
                log::error!("Failed to stop all executables: {e}");
            }
        }
    }

    ui.same_line();

    let running_count = exe_manager.running_count();
    ui.text(format!("Running: {running_count}"));
}

/// Toggles the main window visibility
pub fn toggle_window() {
    IS_WINDOW_OPEN.store(!IS_WINDOW_OPEN.load(Ordering::Relaxed), Ordering::Relaxed);
}
