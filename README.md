## RustifyWLED

_A continuation of my previous project, SpotifyWLED, in Rust._

A not-so-simple script to listen to the currently playing track on your Spotify account and forward it to a WLED (https://github.com/Aircoookie/WLED) LED matrix.

## Features

- polls the Spotify API for your currently playing track
- applies animations according to: playback state (play/pause), the track's tempo and energy, etc.
- *more to be added*

<div align="center">
  
| Play Animation | Pause Animation |
| -------------- | --------------- |
|<img src="./assets/play.gif" width="250" height="250"/> | <img src="./assets/pause.gif" width="250" height="250"/> |

</div>

## Usage

This application requires you to have a Spotify developer account set up, with the client ID and client secret available.

### Configuration

To setup the configuration, copy `config/config.template.toml` into `config/config.toml` and fill required fields.

#### Spotify Client ID/Secret

This is retrieved in one of three ways:
- from environment variables: `RSPOTIFY_CLIENT_ID` and `RSPOTIFY_CLIENT_SECRET`
- from the `config.toml` file
- from CLI prompt upon running


After the configuration is properly setup, the application can be started with:
```
cargo run
```

In order to start playing the animation, go to:
```
<host ip>:8000/start
```

Upon first startup, you will be redirected to Spotify OAuth authentication.

NOTE: `<host ip>` is the IP of the machine you started this app on (as it runs at `0.0.0.0` by default)
