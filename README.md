# ScreenCaster
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

## Useful Resources

* [WebRTC crate](https://github.com/webrtc-rs/webrtc)
* [Ngrok crate](https://ngrok.com/docs/using-ngrok-with/rust/)
* [Druid crate documentation](https://docs.rs/druid/latest/druid/)
* [Tiny Intro to Async/Await Tokio crate for server](https://www.youtube.com/watch?v=T2mWg91sx-o&ab_channel=ManningPublications)
* [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
* [ScreenCasting app in Java](https://github.com/pcyfox/ScreenCasting/tree/master)
* [ScreenGrabber app in Rust](https://github.com/Ieptor/ScreenGrabber)

