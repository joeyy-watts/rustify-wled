use std::net::{UdpSocket, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{cmp, thread};
use std::thread::JoinHandle;
use artnet_protocol::*;
use log::trace;
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
    active_animations: Arc<Mutex<Vec<JoinHandle<()>>>>,
    stop_flag: Arc<AtomicBool>,
    socket: UdpSocket,
}
impl ArtNetController {
    pub fn new() -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Unable to bind to address!");

        Self {
            active_animations: Arc::new(Mutex::new(Vec::new())),
            stop_flag,
            socket,
        }
    }

    ///
    /// Sends animations to the respective device specified in each Animation struct.
    ///
    /// WARNING: before using, the caller must ensure with ArtNetController::any_playing() that no animations are currently playing.
    ///
    pub fn send_animations(&self, animations: Vec<Animation>) {
        // reset stop flag for new animation
        self.stop_flag.store(false, Ordering::Release);

        for animation in animations {
            let local_stop_flag = self.stop_flag.clone();
            let local_socket = self.socket.try_clone().expect("Unable to clone socket!");
            let local_target = animation.target.clone();

            let handle = thread::spawn(move || {
                // for tracking frame sequence
                // all shards within the same frame will have the same sequence number
                let mut sequence_counter: u8 = 0;

                // TODO: transitions
                if !animation.frames_in.is_none() {
                    for frame in animation.frames_in.clone().unwrap().clone() {
                        ArtNetController::send_frame(&local_target, &animation.get_frame_pixels(), frame, sequence_counter, &local_socket);
                    }
                }

                while !local_stop_flag.load(Ordering::Acquire) {
                    for frame in animation.frames_loop.clone() {
                        ArtNetController::send_frame(&local_target, &animation.get_frame_pixels(), frame, sequence_counter, &local_socket);

                        // to allow for termination mid-animation
                        if local_stop_flag.load(Ordering::Acquire) {
                            trace!("Breaking out of animation loop for target: {}", local_target);
                            break;
                        }

                        sequence_counter = sequence_counter.wrapping_add(1);
                    }
                }

                // TODO: transitions
                if !animation.frames_out.is_none() {
                    for frame in animation.frames_out.clone().unwrap().clone() {
                        ArtNetController::send_frame(&local_target, &animation.get_frame_pixels(), frame, sequence_counter, &local_socket);
                    }
                }
            });

            self.active_animations.lock().unwrap().push(handle);
        }
    }

    pub fn stop_animation(&self) {
        if self.any_playing() {
            trace!("Animations are active, setting stop flag to True");
            self.stop_flag.store(true, Ordering::Release);
        } else {
            trace!("No animations are active, stop flag not set");
        }
    }

    /// Checks if any animation threads are still active.
    /// Automatically drops animation threads that have finished from tracker.
    pub fn any_playing(&self) -> bool {
        let mut active_animations_guard = self.active_animations.lock().unwrap();
        // remove threads that have finished
        active_animations_guard.retain(|handle| !handle.is_finished());
        !active_animations_guard.is_empty()
    }

    /// Sends a single frame (or image) to the target device
    ///
    /// `frame` - the frame to be sent
    ///
    fn send_frame(
        target: &String,
        target_size: &u16,
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

    // NOTE: size is the number of pixels in the target device, dimension-agnostic
    fn calculate_sharded_commands(size: &u16, frame: AnimationFrame, mut sequence_counter: u8) -> Vec<Vec<u8>> {
        // or channels per universe
        static CHANNELS_PER_SHARD: u16 = 510;
        static CHANNELS_PER_PIXEL: u16 = 3;

        let mut commands: Vec<Vec<u8>> = Vec::new();

        // calculate number of shards needed
        // total num of channels in frame = (width * height) * (channels per pixel, 3)
        // we can fit only 170 pixels/510 channels in a single universe, even though the max is 512
        // num of shards = ceil(total num of channels / 510)
        let num_shards: u16 = (
            (((size * CHANNELS_PER_PIXEL) as u32) / CHANNELS_PER_SHARD as u32) + 1
        ) as u16;

        for u in 0..num_shards {
            let start: usize = (u * CHANNELS_PER_SHARD) as usize;
            let end: usize = cmp::min(((u + 1) * CHANNELS_PER_SHARD) - 1, (frame.data.len() - 1) as u16) as usize;
            let frame_slice = frame.data[start..=end].to_vec();
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
