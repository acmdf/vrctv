<div align="center">

# <img src="https://raw.githubusercontent.com/ACMDF/VRCTV/master/vrctv-desktop/src/favicon.png" width="64" height="64"> </img> VRCTV

</div>

VRCTV is a stream helper app, that provides avatar changing and overlay reward support

# Building

## Desktop App

When working on the desktop app, all commands must be run in the vrctv-desktop directory. 
To install the dependencies, run `pnpm i`

- To run the app during development, run `pnpm tauri dev`
- To build executables for installation, use `pnpm tauri build`

## Backend server

- To run the app during development, run `just dev`
- To build a production version, use a standard rust build `cargo build --release -p vrctv-server`

# In future

- Testing is non-existent
- Overlay UI could be improved
- Anything your heart desires

# Contributing

If you're interested in contributing your feature or patch to project Lily, simply open a pull request. However, if it's more complex than simple changes, contact me on discord (same username) so we can talk about how it's implemented and if it's within scope.
