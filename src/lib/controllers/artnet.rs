use std::net::{UdpSocket, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{cmp, thread};
use artnet_protocol::*;
use crate::lib::artnet::anim::frame::AnimationFrame;


/// Controller module for 2D (matrix-based) ArtNet devices
/// 
/// A UDPSocket will be connected throughout the lifecycle of the controller.
/// Once destroyed, the UDPSocket connection will be terminated.
/// 
/// `target` - address of the target ArtNet device, without port
/// `dimensions` - height and weight of the target device
/// 
pub struct ArtNetController2D {
    pub inner: Arc<ArtNetController2DInner>,
    pub is_playing: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
} 

pub struct ArtNetController2DInner {
    pub target: String,
    pub dimensions: (u16, u16),
    socket: UdpSocket,
}

impl ArtNetController2D {
    pub fn new(target: String, dimensions: (u16, u16)) -> Self {
        let inner = Arc::new(ArtNetController2DInner::new(target, dimensions));
        let is_playing = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::new(AtomicBool::new(false));

        Self { inner, is_playing, stop_flag }
    }

    pub fn send_frames(&self, frames: Vec<AnimationFrame>, frame_interval: f64) {
        self.is_playing.store(true, Ordering::Relaxed);

        let local_inner = self.inner.clone();
        let local_stop_flag = self.stop_flag.clone();
        let local_playing_flag = self.is_playing.clone();

        thread::spawn(move || {
            let mut sequence_counter: u8 = 0;

            while !local_stop_flag.load(Ordering::Relaxed) {
                for frame in frames.clone() {
                    local_inner.send_frame(frame, sequence_counter);
                    
                    sequence_counter = sequence_counter.wrapping_add(1);
                    
                    thread::sleep(Duration::from_secs_f64(frame_interval));
                }
            }

            local_playing_flag.store(false, Ordering::Relaxed);
        });
    }

    pub fn stop_animation(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

///
/// Inner struct for ArtNetController2D
/// to be used inside a thread
impl ArtNetController2DInner {
    pub fn new(target: String, dimensions: (u16, u16)) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Unable to bind to address!");

        Self { target, dimensions, socket }
    }

    /// Sends a single frame (or image) to the target device
    /// 
    /// `frame` - the frame to be sent
    /// 
    fn send_frame(&self, frame: AnimationFrame, sequence_counter: u8) {
        // or channels per universe
        static CHANNELS_PER_SHARD: u16 = 510;
        static CHANNELS_PER_PIXEL: u16 = 3;

        // calculate number of shards needed
        // total num of channels in frame = (width * height) * (channels per pixel, 3)
        // we can fit only 170 pixels/510 channels in a single universe, even though the max is 512
        // num of shards = ceil(total num of channels / 510)
        let num_shards: u16 = (
            (((self.dimensions.0 * self.dimensions.1 * CHANNELS_PER_PIXEL) as u32) / CHANNELS_PER_SHARD as u32) + 1
        ) as u16;

        let addr = format!("{}:6454", self.target).to_socket_addrs().unwrap().next().unwrap();
        let mut commands = Vec::new();

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
        
        for command_byte in commands {
            self.socket.send_to(&command_byte, &addr).unwrap();
        }


    }
}
