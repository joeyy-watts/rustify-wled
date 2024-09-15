use std::net::{UdpSocket, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{cmp, thread};
use artnet_protocol::*;

use crate::lib::models::animation::Animation;
use crate::lib::models::frame::AnimationFrame;
use crate::settings::SETTINGS;

/// Controller module for ArtNet devices
/// 
/// A UDPSocket will be connected throughout the lifecycle of the controller.
/// Once destroyed, the UDPSocket connection will be terminated.
/// 
/// `target` - address of the target ArtNet device, without port
/// `dimensions` - height and weight of the target device
/// 
pub struct ArtNetController {
    pub is_playing: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
    size: (u8, u8),
    target: String,
    socket: UdpSocket,
}
impl ArtNetController {
    pub fn new(target: String, size: (u8, u8)) -> Self {
        let is_playing = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::new(AtomicBool::new(false));
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Unable to bind to address!");

        Self {
            is_playing,
            stop_flag,
            target,
            size,
            socket,
        }

    }

    pub fn send_animation(&self, animation: Animation) {
        self.is_playing.store(true, Ordering::Relaxed);

        let local_stop_flag = self.stop_flag.clone();
        let local_playing_flag = self.is_playing.clone();
        let local_socket = self.socket.try_clone().expect("Unable to clone socket!");
        let local_target = self.target.clone();
        let local_size = self.size.clone();

        thread::spawn(move || {
            // for tracking frame sequence
            // all shards within the same frame will have the same sequence number
            let mut sequence_counter: u8 = 0;

            // TODO: transitions
            if !animation.frames_in.is_none() {
                for frame in animation.frames_in.unwrap().clone() {
                    ArtNetController::send_frame(&local_target, &local_size, frame, sequence_counter, &local_socket);
                }
            }

            while !local_stop_flag.load(Ordering::Relaxed) {
                for frame in animation.frames_loop.clone() {
                    ArtNetController::send_frame(&local_target, &local_size, frame, sequence_counter, &local_socket);
                    
                    // to allow for termination mid-animation
                    if local_stop_flag.load(Ordering::Relaxed) {
                        break;
                    }

                    sequence_counter = sequence_counter.wrapping_add(1);
                }
            }

            // TODO: transitions
            if !animation.frames_out.is_none() {
                for frame in animation.frames_out.unwrap().clone() {
                    ArtNetController::send_frame(&local_target, &local_size, frame, sequence_counter, &local_socket);
                }
            }

            local_playing_flag.store(false, Ordering::Relaxed);
            local_stop_flag.store(false, Ordering::Relaxed);
        });
    }

    pub fn stop_animation(&self) {
        if self.is_playing.load(Ordering::Relaxed) {
            self.stop_flag.store(true, Ordering::Relaxed);
        }
    }

    /// Sends a single frame (or image) to the target device
    ///
    /// `frame` - the frame to be sent
    ///
    fn send_frame(
        target: &String,
        target_size: &(u8, u8),
        frame: AnimationFrame,
        sequence_counter: u8,
        socket: &UdpSocket,
    ) {
        let addr = format!("{}:6454", target).to_socket_addrs().unwrap().next().unwrap();
        let commands = Self::calculate_sharded_commands(target_size, frame, sequence_counter);

        for command_byte in commands {
            socket.send_to(&command_byte, &addr).unwrap();
        }

        thread::sleep(Duration::from_secs_f64(SETTINGS.read().unwrap().animation.frame_interval));
    }

    fn calculate_sharded_commands(size: &(u8, u8), frame: AnimationFrame, mut sequence_counter: u8) -> Vec<Vec<u8>> {
        // or channels per universe
        static CHANNELS_PER_SHARD: u16 = 510;
        static CHANNELS_PER_PIXEL: u16 = 3;

        let mut commands: Vec<Vec<u8>> = Vec::new();

        // calculate number of shards needed
        // total num of channels in frame = (width * height) * (channels per pixel, 3)
        // we can fit only 170 pixels/510 channels in a single universe, even though the max is 512
        // num of shards = ceil(total num of channels / 510)
        let num_shards: u16 = (
            ((((size.0 as u16) * (size.1 as u16) * CHANNELS_PER_PIXEL) as u32) / CHANNELS_PER_SHARD as u32) + 1
        ) as u16;

        for u in 0..num_shards {
            let start: usize = (u * CHANNELS_PER_SHARD) as usize;
            let end: usize = cmp::min(((u + 1) * CHANNELS_PER_SHARD) - 1, (frame.data.len()) as u16) as usize;
            let frame_slice = frame.data[start..end].to_vec();
            let command: ArtCommand = ArtCommand::Output(Output {
                data: frame_slice.into(), // The data we're sending to the node
                sequence: sequence_counter,
                port_address: PortAddress::try_from(u as u8).unwrap(),
                ..Output::default()
            });

            let bytes = command.write_to_buffer().unwrap();

            commands.push(bytes);
        }

        commands
    }
}
