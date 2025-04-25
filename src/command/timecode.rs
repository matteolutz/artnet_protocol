data_structure! {
    #[derive(Debug)]
    #[doc = "Used to send timecode data"]
    pub struct Timecode {
        #[doc = "Determines which version the server has. Will be ARTNET_PROTOCOL_VERSION by default"]
        pub version: [u8; 2],

        #[doc = "Filler byte"]
        pub filler1: u8,

       #[doc = "Used to identify different streams of time code. Value of 0x00 is the master."]
        pub stream_id: u8,

       #[doc = "Frames time. 0 – 29 depending on mode."]
        pub frames: u8,

        #[doc = "Seconds. 0 - 59."]
        pub seconds: u8,

        #[doc = "Minutes. 0 - 59."]
        pub minutes: u8,

        #[doc = "Hours. 0 - 23."]
        pub hours: u8,

        #[doc = "Timecode key type. 0 = 24, 1 = 25, 2 = 29.97, 3 = 30"]
        pub key_type: u8,
    }
}

impl Default for Timecode {
    fn default() -> Timecode {
        Timecode {
            version: super::ARTNET_PROTOCOL_VERSION,
            filler1: 0,
            stream_id: 0,
            frames: 0,
            seconds: 0,
            minutes: 0,
            hours: 0,
            key_type: 0,
        }
    }
}
