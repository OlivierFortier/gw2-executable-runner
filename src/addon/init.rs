/*!
# Nexus Addon Initialization Module

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
    log::info!("Loading Gw2 Executable Loader addon");

    if let Err(e) = init_addon() {
        log::error!("Failed to initialize nexus addon: {e}");
        return;
    }

    log::info!("Gw2 Executable Loaderr addon loaded successfully");
}

/// Internal initialization function with proper error handling
fn init_addon() -> Result<()> {
    // Initialize the nexus menus and options
    // Create the addon dir if it doesn't exist
    use std::fs;

    let addon_dir = get_addon_dir("Gw2 Executable Loader").ok_or_else(|| {
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

    // Load textures for the addon
    load_addon_textures()?;

    // Setup quick access menu
    setup_quick_access()?;

    // Setup keybinds
    setup_keybinds()?;

    // Setup UI rendering
    ui::setup_main_window_rendering();

    Ok(())
}

/// Loads the addon textures from embedded resources
fn load_addon_textures() -> Result<()> {
    let icon = include_bytes!("../../images/64p_exe_loader.png");
    let icon_hover = include_bytes!("../../images/64p_exe_loader.png");

    let receive_texture: RawTextureReceiveCallback = texture_receive!(|id, _texture| {
        log::info!("texture {id} loaded");
    });

    // Note: load_texture_from_memory doesn't return a Result, so we assume success
    // In a real implementation, we might want to add validation
    load_texture_from_memory("GW2_EXECUTABLE_LOADER_ICON", icon, Some(receive_texture));
    load_texture_from_memory(
        "GW2_EXECUTABLE_LOADER_ICON_HOVER",
        icon_hover,
        Some(receive_texture),
    );

    log::info!("Addon textures loaded successfully");
    Ok(())
}

/// Sets up the quick access menu entry
fn setup_quick_access() -> Result<()> {
    // Note: add_quick_access doesn't return a Result, so we assume success
    // In a real implementation, we might want to add validation
    add_quick_access(
        "GW2_EXECUTABLE_LOADER_SHORTCUT",
        "GW2_EXECUTABLE_LOADER_ICON",
        "GW2_EXECUTABLE_LOADER_ICON_HOVER",
        "GW2_EXECUTABLE_LOADER_KEYBIND",
        "Gw2 executable loader",
    )
    .revert_on_unload();

    log::info!("Quick access menu setup successfully");
    Ok(())
}

/// Sets up the keybind handlers
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

    // Note: register_keybind_with_string doesn't return a Result, so we assume success
    // In a real implementation, we might want to add validation
    register_keybind_with_string(
        "GW2_EXECUTABLE_LOADER_KEYBIND",
        main_window_keybind_handler,
        "ALT+SHIFT+1",
    )
    .revert_on_unload();

    log::info!("Keybinds setup successfully");
    Ok(())
}

/// Nexus addon unload function - handles cleanup of all nexus-specific functionality
pub fn unload() {
    log::info!("Unloading Gw2 executable loader addon");

    if let Err(e) = cleanup_addon() {
        log::error!("Error during nexus addon cleanup: {e}");
    }

    log::info!("Gw2 executable loader addon unloaded");
}

/// Internal cleanup function with proper error handling
fn cleanup_addon() -> Result<()> {
    // Stop all running executables before unloading
    if let Some(exe_manager_arc) = crate::addon::manager::EXE_MANAGER.get() {
        let mut exe_manager = exe_manager_arc.lock().map_err(|e| {
            NexusError::ManagerInitialization(format!(
                "Failed to lock exe manager during cleanup: {e}"
            ))
        })?;
        exe_manager.stop_all()?;
    }

    log::info!("Nexus addon cleanup completed successfully");
    Ok(())
}
