# GW2 Executable Runner - Qwen Context

## Project Overview

This is a Rust-based executable runner for Nexus, designed to work with Blish HUD and other similar tools. The project allows dynamic running and management of executables from the Guild Wars 2 environment, particularly useful for Linux users who want to run custom addons or tools alongside the game.

The addon integrates with the Nexus framework to provide a UI for selecting and managing executables that run within the same Wine prefix and binary as Guild Wars 2 on Linux systems.

## Key Technologies

- **Language**: Rust 2024 edition
- **Framework**: Nexus-rs (https://github.com/zerthox/nexus-rs)
- **UI Library**: ImGui (via Nexus)
- **Build System**: Cargo
- **Dependencies**:
  - `log` for logging
  - `rfd` for file dialogs
  - `sysinfo` for system information
  - `serde` and `serde_json` for serialization
  - `nexus` for integration with the Nexus framework

## Architecture

### Main Components

1. **Library Entry Point** (`src/lib.rs`):
   - Defines the addon metadata and exports
   - Links to the addon module

2. **Addon Module** (`src/addon/`):
   - **Initialization** (`init.rs`): Handles addon loading, resource loading, UI setup, keybinds, and cleanup
   - **Manager** (`manager.rs`): Core logic for managing executable paths, launching/stopping processes, and persistence
   - **UI** (`ui.rs`): Renders the ImGui interface for managing executables
   - **Error Handling**: Custom `NexusError` enum and `Result` type alias

### Key Features

- Dynamic running and initialization of executables inside the GW2 environment
- UI for selecting and managing executables
- Persistent storage of executable paths in `exes.txt`
- Process tracking and cleanup
- Keybind support (ALT+SHIFT+2 by default)
- Quick access menu integration

## Building and Running

### Prerequisites

- Rust toolchain (edition 2024)
- Cargo package manager

### Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# The build outputs will be in:
# target/debug/gw2_executable_runner.dll (development)
# target/x86_64-pc-windows-msvc/release/gw2_executable_runner.dll (release)
```

### Installation and Usage

1. Download the DLL from releases or build it yourself
2. Place the DLL in the `addons` directory of your Guild Wars 2 folder
3. Start the game and enable the addon in the Nexus settings
4. Click on the addon icon in the Nexus UI to open the executable loader interface
5. Browse for executables and manage them through the UI

### Keybinds

- **ALT+SHIFT+2**: Toggle the main addon window

## Development Conventions

### Code Structure

- Modular design with separate modules for initialization, management, and UI
- Consistent error handling using the custom `NexusError` enum
- Logging throughout the application using the `log` crate
- UI rendering separated from business logic

### Error Handling

- Custom `NexusError` enum with variants for different error types:
  - `ManagerInitialization`
  - `ProcessLaunch`
  - `ProcessStop`
  - `FileOperation`
  - `ResourceLoading`
- All fallible operations return `Result<T, NexusError>`

### UI Architecture

- Uses ImGui through Nexus framework
- Main window rendering is registered during addon initialization
- State management using atomic booleans for window visibility
- Separation of UI rendering logic from business logic

### Persistence

- Executable paths are stored in `exes.txt` in the addon directory
- Automatic loading and saving of executable lists
- Settings are saved when modified

## Testing

Currently, there are no explicit testing commands defined in the project configuration. Testing would involve:

1. Building the addon
2. Installing it in a Guild Wars 2 environment with Nexus
3. Manually verifying functionality through the UI

## Continuous Integration

The project uses GitHub Actions for building releases:

- Workflow defined in `.github/workflows/build.yaml`
- Builds on Windows with the `x86_64-pc-windows-msvc` target
- Automatically creates releases when tags are pushed
- Uploads both DLL and PDB files as artifacts

## Project Status

This is a functional addon with a stable API. The main features are implemented:
- Executable management
- Process launching and tracking
- UI interface
- Persistence
- Proper cleanup on unload

The project follows semantic versioning with the current version at 0.1.1.