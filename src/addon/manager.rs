/*!
# Executable Manager Module

Handles all executable management functionality ,including:
- Persistent storage of executable paths
- Launching and stopping processes
- Process tracking and cleanup
- File dialog integration for selecting executables

## Usage Example

```rust
use crate::addon::manager::ExeManager;
use std::path::PathBuf;

let addon_dir = PathBuf::from("path/to/addon");
let mut manager = ExeManager::new(addon_dir)?;

// Add an executable
manager.add_exe("C:\\Windows\\System32\\notepad.exe".to_string())?;

// Launch an executable
manager.launch_exe("C:\\Windows\\System32\\notepad.exe")?;

// Stop all running executables
manager.stop_all()?;
```

## Error Handling

All fallible operations return `Result<T, NexusError>`. Errors are logged using the `log` crate.

*/

use std::{
    collections::HashMap,
    fs::{read_to_string, write},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
};

use crate::addon::{NexusError, Result};

/**
 * Manages executable files and their running processes.
 *
 * Stores a list of executable paths, tracks running processes, and provides methods for launching, stopping,
 * and cleaning up executables. All operations return a `Result<T, NexusError>` for robust error handling.
 */
#[derive(Debug)]
pub struct ExeManager {
    exe_paths: Vec<String>,
    running_processes: HashMap<String, Child>,
    addon_dir: PathBuf,
}

impl ExeManager {
    /**
     * Creates a new ExeManager instance and loads the existing exe list from disk.
     *
     * # Arguments
     * * `addon_dir` - Path to the addon directory containing exes.txt
     *
     * # Errors
     * Returns `NexusError::FileOperation` if loading the exe list fails.
     */
    pub fn new(addon_dir: PathBuf) -> Result<Self> {
        let mut manager = Self {
            exe_paths: Vec::new(),
            running_processes: HashMap::new(),
            addon_dir,
        };
        manager.load_exe_list()?;
        Ok(manager)
    }

    /**
     * Loads the executable list from the exes.txt file in the addon directory.
     *
     * # Errors
     * Returns `NexusError::FileOperation` if reading the file fails.
     */
    fn load_exe_list(&mut self) -> Result<()> {
        let mut exes_file = self.addon_dir.clone();
        exes_file.push("exes.txt");

        match read_to_string(&exes_file) {
            Ok(contents) => {
                self.exe_paths = contents
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| line.trim().to_string())
                    .collect();
                log::info!("Loaded {} executables from exe list", self.exe_paths.len());
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::info!("No existing exe list found, starting with empty list");
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to read exe list from {exes_file:?}: {e}");
                log::error!("{error_msg}");
                Err(NexusError::FileOperation(error_msg))
            }
        }
    }

    /**
     * Saves the current executable list to the exes.txt file.
     *
     * # Errors
     * Returns `NexusError::FileOperation` if writing to the file fails.
     */
    fn save_exe_list(&self) -> Result<()> {
        let mut exes_file = self.addon_dir.clone();
        exes_file.push("exes.txt");

        let content = self.exe_paths.join("\n");
        write(&exes_file, content).map_err(|e| {
            let error_msg = format!("Failed to save exe list to {exes_file:?}: {e}");
            log::error!("{error_msg}");
            NexusError::FileOperation(error_msg)
        })?;

        log::debug!("Saved {} executables to exe list", self.exe_paths.len());
        Ok(())
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

        if self.exe_paths.contains(&path) {
            log::warn!("Executable path already exists: {path}");
            return Ok(());
        }

        self.exe_paths.push(path.clone());
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
        if index >= self.exe_paths.len() {
            return Err(NexusError::FileOperation(format!(
                "Invalid index {} for exe list of length {}",
                index,
                self.exe_paths.len()
            )));
        }

        let path = self.exe_paths.remove(index);

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

        for (path, mut child) in self.running_processes.drain() {
            if let Err(e) = child.kill() {
                let error_msg = format!("Failed to stop {path}: {e}");
                log::error!("{error_msg}");
                errors.push(error_msg);
            } else {
                log::info!("Stopped executable: {path}");
            }
        }

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
     * Gets a reference to the exe paths list.
     *
     * # Returns
     * Reference to the vector of executable paths.
     */
    pub fn exe_paths(&self) -> &Vec<String> {
        &self.exe_paths
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
