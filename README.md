<div align="center">

# <img src="https://raw.githubusercontent.com/acmdf/vrctv/main/vrctv-desktop/src/favicon.png" width="64" height="64"> </img> VRCTV

[![GitHub release](https://img.shields.io/github/release/acmdf/vrctv.svg)](https://github.com/acmdf/vrctv/releases/latest)
[![VRCX Discord Invite](https://img.shields.io/badge/discord-invite-blue?logo=discord)](https://discord.gg/tvuPEkQrW9)

</div>

VRCTV is a stream helper app, that provides avatar changing and overlay reward support

## Current Features

### Avatars

- Change avatar, switching back after a time delay to either a specific avatar or the previously detected one
- Change OSC Parameters on a certain avatar, such as adding or removing elements of the avi, optionally reverting them after

### Wardudo

- Send OSC messages to [Warudo OSC](https://steamcommunity.com/sharedfiles/filedetails/?id=3006445377), which can be used to toggle expressions and objects, optionally reverting over a period of time

### General Overlay

- Change the visibility of browser elements in OBS based on triggers, such as showing images on stream or enabling and disabling streamlabs elements

# Testing

To test VRCTV simply download one of the releases and it should automatically connect, you then need both the following avatars in your favourited avatars, and then you can test the `!MaidMode`, `!FurryMode` and `!HideLogo` commands in chat:

- Maid mode: https://vrchat.com/home/avatar/avtr_de75efc5-c67c-4ae8-8a14-cafa07d0fcad
- Furry mode: https://vrchat.com/home/avatar/avtr_da58f525-347c-4be7-8a26-9dc0ebc83782

# Setting up environment

The server component needs some env variables set for it to function correctly, and those can either be set using a .env file in the root directory, or simply setting them in the environment. Following is an example .env file with some good defaults (*Important Note*: changing the scope variables could cause issues, not all of them are used currently but they're set aside for future use):
```
STREAMLABS_REDIRECT=http://localhost:3000/streamlabs/callback
STREAMLABS_SCOPES="donations.read socket.token mediashare.control"
STREAMLABS_CLIENT=my-streamlabs-client-key
STREAMLABS_SECRET=STREAMLABSSECRET

TWITCH_REDIRECT=http://localhost:3000/twitch/callback
TWITCH_SCOPES="user_read bits:read channel:bot channel:read:polls channel:manage:polls channel:read:redemptions channel:manage:redemptions user:read:chat user:read:whispers"
TWITCH_CLIENT=mytwitchclient
TWITCH_SECRET=mytwitchsecret

PUBLIC_WEBSOCKET_URL=wss://example.com/ws
PUBLIC_BACKEND_URL=https://example.com/

CLIENT_VERSION=0.3.1
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
