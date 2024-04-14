pub mod lib {
        pub mod artnet {
            pub mod anim {
                pub mod effects {
                    pub mod playback;
                    pub mod transitions;
                    pub mod waveforms {
                        pub mod waveform;
                        pub mod waveform_impl;
                    }
                    pub mod base {
                        pub mod math;
                        pub mod overlay;
                        pub mod effect;
                    }
            }
            pub mod animation;
            pub mod frame;
        }
    }
    pub mod controllers {
        pub mod animation;
        pub mod artnet;
        pub mod spotify;
        pub mod app;
    }
    pub mod web {
        pub mod webserver;
    }
}

pub mod utils {
    pub mod image;
    pub mod spotify;
}
