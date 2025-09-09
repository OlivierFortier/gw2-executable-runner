/*!

Handles initialization, resource loading, and cleanup for addon.
Provides the main entry points for the addon lifecycle and orchestrates setup of UI, keybinds, quick access, and textures.


*/

use nexus::{
    keybind::register_keybind_with_string,
    keybind_handler,
    paths::get_addon_dir,
    quick_access::add_quick_access,
    texture::{RawTextureReceiveCallback, load_texture_from_memory},
    texture_receive,
};

use crate::addon::{NexusError, Result, manager::ExeManager, ui};

/// Nexus addon load function - handles initialization of all nexus-specific functionality
pub fn load() {
    log::info!("Loading Gw2 Executable Runner addon");

    if let Err(e) = init_addon() {
        log::error!("Failed to initialize Gw2 Executable Runner: {e}");
        return;
    }

    log::info!("Gw2 Executable Runner addon loaded successfully");
}

fn init_addon() -> Result<()> {
    // Initialize the nexus menus and options
    // Create the addon dir if it doesn't exist
    use std::fs;

    let addon_dir = get_addon_dir("gw2_executable_runner").ok_or_else(|| {
        NexusError::ManagerInitialization("Failed to get addon directory".to_string())
    })?;

    fs::create_dir_all(&addon_dir).map_err(|e| {
        NexusError::ManagerInitialization(format!("Failed to create addon directory: {e}"))
    })?;

    // Initialize the exe manager
    let exe_manager = std::sync::Arc::new(std::sync::Mutex::new(ExeManager::new(addon_dir)?));

    crate::addon::manager::EXE_MANAGER
        .set(exe_manager.clone())
        .map_err(|_| {
            NexusError::ManagerInitialization("Failed to set global exe manager".to_string())
        })?;

    load_addon_textures()?;
    setup_quick_access()?;
    setup_keybinds()?;
    ui::setup_main_window_rendering();

    // Launch executables that should start on addon load
    let exe_manager_arc =
        crate::addon::manager::EXE_MANAGER
            .get()
            .ok_or(NexusError::ManagerInitialization(
                "EXE_MANAGER not set during init".to_string(),
            ))?;
    let mut exe_manager = exe_manager_arc.lock().map_err(|e| {
        NexusError::ManagerInitialization(format!(
            "Failed to lock exe manager during startup launch: {e}"
        ))
    })?;

    let paths_to_launch: Vec<String> = exe_manager
        .executables()
        .iter()
        .filter(|exe| exe.launch_on_startup && !exe.is_running)
        .map(|exe| exe.path.clone())
        .collect();

    for path in paths_to_launch {
        if let Err(e) = exe_manager.launch_exe(&path) {
            log::warn!("Failed to launch startup executable {}: {}", path, e);
        } else {
            log::info!("Launched startup executable: {}", path);
        }
    }

    Ok(())
}

/// Loads the addon textures from embedded resources
fn load_addon_textures() -> Result<()> {
    let icon = include_bytes!("../../images/64p_exe_loader.png");
    let icon_hover = include_bytes!("../../images/64p_exe_loader.png");

    let receive_texture: RawTextureReceiveCallback = texture_receive!(|id, _texture| {
        log::info!("texture {id} loaded");
    });

    load_texture_from_memory("GW2_EXECUTABLE_RUNNER_ICON", icon, Some(receive_texture));
    load_texture_from_memory(
        "GW2_EXECUTABLE_RUNNER_ICON_HOVER",
        icon_hover,
        Some(receive_texture),
    );

    log::info!("Addon textures loaded successfully");
    Ok(())
}

fn setup_quick_access() -> Result<()> {
    add_quick_access(
        "GW2_EXECUTABLE_RUNNER_SHORTCUT",
        "GW2_EXECUTABLE_RUNNER_ICON",
        "GW2_EXECUTABLE_RUNNER_ICON_HOVER",
        "GW2_EXECUTABLE_RUNNER_KEYBIND",
        "Gw2 executable runner",
    )
    .revert_on_unload();

    log::info!("Quick access menu setup successfully");
    Ok(())
}

fn setup_keybinds() -> Result<()> {
    let main_window_keybind_handler = keybind_handler!(|id, is_release| {
        log::info!(
            "keybind {id} {}",
            if is_release { "released" } else { "pressed" }
        );
        if !is_release {
            ui::toggle_window();
        }
    });

    register_keybind_with_string(
        "GW2_EXECUTABLE_RUNNER_KEYBIND",
        main_window_keybind_handler,
        "ALT+SHIFT+2",
    )
    .revert_on_unload();

    log::info!("Keybinds setup successfully");
    Ok(())
}

pub fn unload() {
    log::info!("Unloading Gw2 executable runner");

    if let Err(e) = (|| -> Result<()> {
        // Stop all running executables before unloading
        if let Some(exe_manager_arc) = crate::addon::manager::EXE_MANAGER.get() {
            let mut exe_manager = exe_manager_arc.lock().map_err(|e| {
                NexusError::ManagerInitialization(format!(
                    "Failed to lock exe manager during cleanup: {e}"
                ))
            })?;
            exe_manager.stop_all()?;
        }

        log::info!("Gw2 executable runner cleanup completed successfully");
        Ok(())
    })() {
        log::error!("Error during gw2 executable runner cleanup: {e}");
    }

    log::info!("Gw2 executable runner unloaded");
}
