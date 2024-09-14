use std::sync::mpsc::{self, Receiver, Sender};

use crate::lib::controllers::{animation::AnimationControllerMessage, spotify::SpotifyControllerMessage};

use super::playback_state::PlaybackState;


pub struct AppChannels {
    pub sp_msg_tx: Sender<SpotifyControllerMessage>,
    pub sp_msg_rx: Receiver<SpotifyControllerMessage>,
    pub playback_tx: Sender<PlaybackState>,
    pub playback_rx: Receiver<PlaybackState>,
    pub anim_msg_tx: Sender<AnimationControllerMessage>,
    pub anim_msg_rx: Receiver<AnimationControllerMessage>,
}


impl AppChannels {
    /// setup channel for communication between controllers
    /// 
    /// Naming convention is (foo_rx, foo_tx) where `foo` is shorthand for the type being sent
    pub fn setup() -> Self {
        // app.rs -> spotify.rs
        let (sp_msg_tx, sp_msg_rx) = mpsc::channel();
        
        // spotify.rs -> app.rs
        let (playback_tx, playback_rx) = mpsc::channel();

        // app.rs -> animation.rs
        let (anim_msg_tx, anim_msg_rx) = mpsc::channel();

        Self {
            sp_msg_tx,
            sp_msg_rx,
            playback_tx,
            playback_rx,
            anim_msg_tx,
            anim_msg_rx,
        }
    }
}