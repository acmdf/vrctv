<div align="center">

# <img src="https://raw.githubusercontent.com/acmdf/vrctv/main/vrctv-desktop/src/favicon.png" width="64" height="64"> </img> VRCTV

[![GitHub release](https://img.shields.io/github/release/acmdf/vrctv.svg)](https://github.com/acmdf/vrctv/releases/latest)
[![VRCX Discord Invite](https://img.shields.io/badge/discord-invite-blue?logo=discord)](https://discord.gg/tvuPEkQrW9)

</div>

VRCTV is a stream helper app, that provides avatar changing and overlay reward support

# Setting up environment

The server component needs some env variables set for it to function correctly, and those can either be set using a .env file in the root directory, or simply setting them in the environment. Following is an example .env file with some good defaults (*Important Note*: changing the scope variables could cause issues, not all of them are used currently but they're set aside for future use):
```
STREAMLABS_REDIRECT=http://localhost:3000/streamlabs/callback
STREAMLABS_SCOPES="donations.read socket.token mediashare.control"
STREAMLABS_VERSION=1
STREAMLABS_CLIENT=my-streamlabs-client-key
STREAMLABS_SECRET=STREAMLABSSECRET

TWITCH_REDIRECT=http://localhost:3000/twitch/callback
TWITCH_SCOPES="user_read bits:read channel:bot channel:read:polls channel:manage:polls channel:read:redemptions channel:manage:redemptions user:read:chat user:read:whispers"
TWITCH_VERSION=1
TWITCH_CLIENT=mytwitchclient
TWITCH_SECRET=mytwitchsecret

PUBLIC_WEBSOCKET_URL=wss://example.com/ws
PUBLIC_BACKEND_URL=https://example.com/

CLIENT_VERSION=0.2.0
```

# Building

## Desktop App

When working on the desktop app, all commands must be run in the vrctv-desktop directory. 
To install the dependencies, run `pnpm i`

- To run the app during development, run `pnpm tauri dev`
- To build executables for installation, use `pnpm tauri build`

## Backend server

- To run the app during development, run `systemfd --no-pid -s http::3000 -- cargo watch --ignore '*.sqlite' -x "run -p vrctv-server"`
- To build a production version, use a standard rust build `cargo build --release -p vrctv-server`

# In future

- Github releases (+ server selection)
- Audio rewards
- Testing is non-existent
- Polling maybe
- Anything your heart desires

# Contributing

If you're interested in contributing your feature or patch to VRCTV, simply open a pull request. However, if it's more complex than simple changes, contact me on discord (same username) so we can talk about how it's implemented and if it's within scope.
