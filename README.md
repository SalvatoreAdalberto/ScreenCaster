# ScreenCaster
### by Salvatore Adalberto Esposito s304800 and Marco Del Core s322783
# Compilation and Execution of the Application

To compile the project, execute the respective script based on your operating system:

- On **Linux/MacOS**, run the `launcher.sh` script.
- On **Windows**, execute the `launcher.bat` script.
  
Both accept as parameter a boolean: set `true` if you want to cargo clean your project before compiling, or set `false`if you want to stay with a previously compiled version (in any case `screen_caster`will be compiled, as any missing module).

These scripts perform the following operations:

1. **Compilation of All Modules**:
   - The scripts ensure that each of the required modules is compiled, including:
     - **screen_caster**
     - **annotation_tool**
     - **overlay_crop**
     - **setup**

2. **Application Launch**:
   - Upon successful compilation of all modules, `setup` module is run to check if `ffmpeg` is alrready present on your device, otherwise a proper version is downloaded and intalled. Then
  the application is initialized by launching the `screen_caster` module as the main application.

**Important**: The main application (`screen_caster`) cannot be executed unless all other dependent modules are correctly configured and compiled. These scripts ensure the proper configuration and compilation of each module before the application is launched, thereby preventing any runtime errors due to missing or misconfigured components.

---

# Creating a Screencasting Application in Rust

Create a screencasting application using Rust that continuously captures screen content (or a portion thereof) and streams it to peers. The application should meet the following requirements:

## Requirements

1. **Platform Support**: Ensure compatibility with Windows, macOS, and Linux.
2. **User Interface (UI)**: Design an intuitive and user-friendly interface for easy navigation.
3. **Operating Mode**: Allow users to select between caster and receiver modes at startup. Receivers should specify the caster's address.
4. **Selection Options**: Enable area restriction for captured content in casting mode.
5. **Hotkey Support**: Implement customizable keyboard shortcuts for pausing/resuming transmission, blanking the screen, and ending sessions.

## Bonus Features

* **Annotation Tools**: Offer tools for overlaying annotations on captured content in casting mode.
* **Save Options**: Allow receivers to record received content to video files.
* **Multi-monitor Support**: Recognize and handle multiple monitors independently for content casting.