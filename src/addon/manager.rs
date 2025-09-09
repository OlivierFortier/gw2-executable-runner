/*!
# Executable Manager Module

Handles all executable management functionality ,including:
- Persistent storage of executable paths
- Launching and stopping processes
- Process tracking and cleanup
- File dialog integration for selecting executables

*/

use std::{
    collections::HashMap,
    fs::{read_to_string, write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::addon::{NexusError, Result};

/// Stores a list of executable paths, tracks running processes, and provides methods for launching, stopping,
/// and cleaning up executables. All operations return a `Result<T, NexusError>`.
/// Executable list is persisted in JSON format in the addon directory.
#[derive(Debug)]
pub struct ExeManager {
    running_processes: HashMap<String, Child>,
    addon_dir: PathBuf,
    executables: Vec<Executable>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Executable {
    pub path: String,
    pub launch_on_startup: bool,
    #[serde(skip)]
    pub is_running: bool,
}

impl ExeManager {
    /// Creates a new ExeManager instance and loads the existing exe list from disk.
    ///
    /// # Arguments
    /// * `addon_dir` - Path to the addon directory containing exes.json
    ///
    /// # Errors
    /// Returns `NexusError::FileOperation` if loading the exe list fails.
    pub fn new(addon_dir: PathBuf) -> Result<Self> {
        let mut manager = Self {
            running_processes: HashMap::new(),
            addon_dir,
            executables: Vec::new(),
        };
        manager.load_exe_list()?;
        Ok(manager)
    }

    pub fn executables(&self) -> &Vec<Executable> {
        &self.executables
    }

    /// Loads the executable list from the exes.json file in the addon directory.
    ///
    /// # Errors
    /// Returns `NexusError::FileOperation` if reading the file fails.
    fn load_exe_list(&mut self) -> Result<()> {
        let mut exes_file = self.addon_dir.clone();
        exes_file.push("exes.json");

        match read_to_string(&exes_file) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(executables) => {
                    self.executables = executables;
                    log::info!(
                        "Loaded {} executables from exe list",
                        self.executables.len()
                    );
                    Ok(())
                }
                Err(e) => {
                    let error_msg = format!("Failed to parse exe list from {:?}: {}", exes_file, e);
                    log::error!("{}", error_msg);
                    Err(NexusError::FileOperation(error_msg))
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::info!("No existing exe list found, starting with empty list");
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to read exe list from {:?}: {}", exes_file, e);
                log::error!("{}", error_msg);
                Err(NexusError::FileOperation(error_msg))
            }
        }
    }

    /// Saves the current executable list to the exes.json file.
    ///
    /// # Errors
    /// Returns `NexusError::FileOperation` if writing to the file fails.
    fn save_exe_list(&self) -> Result<()> {
        let mut exes_file = self.addon_dir.clone();
        exes_file.push("exes.json");

        match serde_json::to_string_pretty(&self.executables) {
            Ok(content) => {
                write(&exes_file, content).map_err(|e| {
                    let error_msg = format!("Failed to save exe list to {:?}: {}", exes_file, e);
                    log::error!("{}", error_msg);
                    NexusError::FileOperation(error_msg)
                })?;
                log::debug!("Saved {} executables to exe list", self.executables.len());
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to serialize exe list: {}", e);
                log::error!("{}", error_msg);
                Err(NexusError::FileOperation(error_msg))
            }
        }
    }

    /**
     * Adds a new executable path to the list and persists it.
     *
     * # Arguments
     * * `path` - Path to the executable file
     *
     * # Errors
     * Returns `NexusError::FileOperation` if the path is empty or saving fails.
     */
    pub fn add_exe(&mut self, path: String) -> Result<()> {
        if path.trim().is_empty() {
            return Err(NexusError::FileOperation(
                "Cannot add empty executable path".to_string(),
            ));
        }

        if self.executables.iter().any(|exe| exe.path == path) {
            log::warn!("Executable path already exists: {path}");
            return Ok(());
        }

        self.executables.push(Executable {
            path: path.clone(),
            launch_on_startup: false,
            is_running: false,
        });
        self.save_exe_list()?;
        log::info!("Added executable: {path}");
        Ok(())
    }

    /**
     * Removes an executable from the list by index and stops its process if running.
     *
     * # Arguments
     * * `index` - Index of the executable in the list
     *
     * # Errors
     * Returns `NexusError::FileOperation` if the index is invalid or saving fails.
     */
    pub fn remove_exe(&mut self, index: usize) -> Result<()> {
        if index >= self.executables.len() {
            return Err(NexusError::FileOperation(format!(
                "Invalid index {} for exe list of length {}",
                index,
                self.executables.len()
            )));
        }

        let path = self.executables.remove(index).path;

        // Kill the process if it's running
        if let Some(mut child) = self.running_processes.remove(&path) {
            if let Err(e) = child.kill() {
                log::warn!("Failed to kill process for removed executable {path}: {e}");
            } else {
                log::info!("Stopped process for removed executable: {path}");
            }
        }

        self.save_exe_list()?;
        log::info!("Removed executable: {path}");
        Ok(())
    }

    /**
     * Launches an executable by path.
     *
     * # Arguments
     * * `path` - Path to the executable file
     *
     * # Errors
     * Returns `NexusError::ProcessLaunch` if the process is already running or spawning fails.
     */
    pub fn launch_exe(&mut self, path: &str) -> Result<()> {
        use std::os::windows::process::CommandExt;

        if self.running_processes.contains_key(path) {
            return Err(NexusError::ProcessLaunch(format!(
                "Process is already running: {path}"
            )));
        }

        // Update the is_running flag in the executables vector
        if let Some(executable) = self.executables.iter_mut().find(|exe| exe.path == path) {
            executable.is_running = true;
        }

        match Command::new(path)
            .creation_flags(0x08000000)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                log::info!("Launched executable: {path}");
                self.running_processes.insert(path.to_string(), child);
                Ok(())
            }
            Err(e) => {
                // Reset the is_running flag on failure
                if let Some(executable) = self.executables.iter_mut().find(|exe| exe.path == path) {
                    executable.is_running = false;
                }
                let error_msg = format!("Failed to launch {path}: {e}");
                log::error!("{error_msg}");
                Err(NexusError::ProcessLaunch(error_msg))
            }
        }
    }

    /**
     * Stops a running executable by path.
     *
     * # Arguments
     * * `path` - Path to the executable file
     *
     * # Errors
     * Returns `NexusError::ProcessStop` if the process is not running or killing fails.
     */
    pub fn stop_exe(&mut self, path: &str) -> Result<()> {
        // Reset the is_running flag in the executables vector
        if let Some(executable) = self.executables.iter_mut().find(|exe| exe.path == path) {
            executable.is_running = false;
        }

        if let Some(mut child) = self.running_processes.remove(path) {
            match child.kill() {
                Ok(_) => {
                    log::info!("Stopped executable: {path}");
                    Ok(())
                }
                Err(e) => {
                    let error_msg = format!("Failed to stop {path}: {e}");
                    log::error!("{error_msg}");
                    Err(NexusError::ProcessStop(error_msg))
                }
            }
        } else {
            Err(NexusError::ProcessStop(format!(
                "Process is not running: {path}"
            )))
        }
    }

    /**
     * Cleans up finished processes from the running processes map.
     * Should be called periodically to avoid resource leaks.
     */
    pub fn cleanup_finished_processes(&mut self) {
        let mut finished = Vec::new();

        for (path, child) in &mut self.running_processes {
            if let Ok(Some(_)) = child.try_wait() {
                finished.push(path.clone());
            }
        }

        for path in finished {
            self.running_processes.remove(&path);
            // Reset the is_running flag in the executables vector
            if let Some(executable) = self.executables.iter_mut().find(|exe| exe.path == path) {
                executable.is_running = false;
            }
            log::info!("Process finished: {path}");
        }
    }

    /**
     * Checks if an executable is currently running.
     *
     * # Arguments
     * * `path` - Path to the executable file
     *
     * # Returns
     * `true` if the process is running, `false` otherwise.
     */
    pub fn is_running(&self, path: &str) -> bool {
        // First check the executables vector for the is_running flag
        if let Some(executable) = self.executables.iter().find(|exe| exe.path == path) {
            if executable.is_running {
                return true;
            }
        }
        // Fallback to checking the running_processes map
        self.running_processes.contains_key(path)
    }

    /**
     * Stops all running executables.
     *
     * # Errors
     * Returns `NexusError::ProcessStop` if any process fails to stop.
     */
    pub fn stop_all(&mut self) -> Result<()> {
        let mut errors = Vec::new();

        // Reset all is_running flags in the executables vector
        log::info!("Resetting is_running flags for all {} executables", self.executables.len());
        for executable in &mut self.executables {
            executable.is_running = false;
        }
        log::info!("Finished resetting is_running flags");

        log::info!("Starting to stop {} running processes", self.running_processes.len());
        for (path, mut child) in self.running_processes.drain() {
            log::info!("Attempting to stop process for path: '{}' with PID: {}", path, child.id());
            if let Err(e) = child.kill() {
                let error_msg = format!("Failed to stop {path}: {e}");
                log::error!("{error_msg} (PID: {})", child.id());
                errors.push(error_msg);
            } else {
                log::info!("Successfully stopped executable: '{}' (PID: {})", path, child.id());
            }
        }
        log::info!("Finished stopping all processes");

        if !errors.is_empty() {
            return Err(NexusError::ProcessStop(format!(
                "Failed to stop some processes: {}",
                errors.join(", ")
            )));
        }

        log::info!("Successfully stopped all running executables");
        Ok(())
    }

    /**
     * Gets the number of running processes.
     *
     * # Returns
     * Number of currently running processes.
     */
    pub fn running_count(&self) -> usize {
        self.running_processes.len()
    }

    pub(crate) fn save_settings(&self) -> Result<()> {
        self.save_exe_list()
    }

    pub(crate) fn launch_on_startup(&mut self, index: usize) -> &mut bool {
        if index >= self.executables.len() {
            panic!(
                "Index out of bounds: {} >= {}",
                index,
                self.executables.len()
            );
        }
        &mut self.executables[index].launch_on_startup
    }
}

/// Opens a file dialog to select an executable file
pub fn open_file_dialog() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Executable Files", &["exe"])
        .add_filter("All Files", &["*"])
        .set_title("Select Executable")
        .pick_file()
        .map(|path| path.to_string_lossy().to_string())
}

/// Global static reference to the exe manager
pub static EXE_MANAGER: std::sync::OnceLock<Arc<Mutex<ExeManager>>> = std::sync::OnceLock::new();
