## RustifyWLED

A continuation of my previous project, SpotifyWLED, in Rust.

This web application is intended to be used with a WLED LED matrix device.

## Features

- polls the Spotify API for your currently playing track
- applies animations according to: playback state (play/pause), the track's tempo and energy, etc.
- *more to be added*

## Usage

*a lot of things are currently hardcoded, I will make proper configs eventually..*

This application requires you to have a Spotify developer account set up, with the client ID and client secret available.

Before starting the application, you are required to set the following environment variables:
```
export RSPOTIFY_CLIENT_ID=<your client id>
export RSPOTIFY_CLIENT_SECRET=<your client secret>
```

Then, the application can be started with:
```
cargo run
```

In order to start playing the animation, go to:
```
<host ip>:8000/start
```

Upon first startup, you will be redirected to Spotify OAuth authentication.

NOTE: `<host ip>` is the IP of the machine you started this app on (as it runs at `0.0.0.0` by default)
