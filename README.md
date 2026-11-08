# Aevum
*Work in progress*

A simple weekly planner with time boxing, written in Rust and Slint.


## About

Aevum is a weekly planner designed to help organize tasks and manage time more efficiently.

The project is still in development, and the user interface is not finished yet.

Aevum is a Latin word meaning "age", "lifetime" or "eternity", referring to the concept of time.

## Rust backend

The Rust backend is mostly finished and can already be used as a standalone API, independently of the user interface.

To demonstrate how it works and test its functionality, I wrote several unit tests. They can be found at the end of `src/task.rs` and `src/time.rs`.

You can run them by executing:

```bash
cargo test
```

## Slint user interface

I chose Slint because I wanted a lightweight, cross-platform framework that allows the application to run on desktop systems and Android devices.

The user interface is still under development.

## Planned features

These are the features I plan to implement in the finished application:

- **Drag & Drop:** Task blocks will appear in a panel on the left and can simply be dragged onto the weekly grid.
- **Custom categories and colors:** Organize tasks into custom categories and assign individual colors to them.
- **Recurring tasks:** Set tasks to repeat daily, weekly or on specific weekdays, optionally at the same time.
- **Task details:** Add optional descriptions and specify only a day if you are not sure when exactly to schedule a task.
- **Data management:** Use a proper database, such as SQLite, to store all data in a single file.
- **Synchronization:** Keep the data file easy to synchronize between devices.
- **Mobile app:** I definitely want to create a well-optimized Android version for both phones and tablets. iOS support is still undecided.
- **Settings:** Provide extensive customization options, including UI colors, sizes and spacing.
- **Feature tour:** Add an optional guide to help users discover the application's features and understand where to find them.

## Installation

Download the latest version for your platform from the [Releases](../../releases) page.

## Building from Source

### Requirements

- Rust and Cargo
- A compatible operating system and the required Slint dependencies

### Build and run

To run the application from source, clone the repository and execute:

```bash
git clone https://github.com/schabbasam/aevum.git
cd aevum
cargo run --release
```

## Web preview

A web preview is also available: link

## License

Feel free to use, modify and distribute this software under the terms of its license, provided that you give appropriate credit to the original authors.

See the [LICENSE](LICENSE) file for the full license text.

Please also refer to the [Slint licensing information](https://slint.dev/pricing) regarding Slint's license terms.
