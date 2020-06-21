use crate::{command::ARTNET_PROTOCOL_VERSION, convert::Convertable, Error, PortAddress, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;

data_structure! {
    #[derive(Debug)]
    #[doc = "ArtDmx is the data packet used to transfer DMX512 data. The format is identical for Node to Controller, Node to Node and Controller to Node."]
    #[doc = ""]
    #[doc = "The Data is output through the DMX O/P port corresponding to the Universe setting. In the absence of received ArtDmx packets, each DMX O/P port re-transmits the same frame continuously. "]
    #[doc = ""]
    #[doc = "The first complete DMX frame received at each input port is placed in an ArtDmx packet as above and transmitted as an ArtDmx packet containing the relevant Universe parameter. Each subsequent DMX frame containing new data (different length or different contents) is also transmitted as an ArtDmx packet."]
    #[doc = ""]
    #[doc = "Nodes do not transmit ArtDmx for DMX512 inputs that have not received data since power on."]
    #[doc = ""]
    #[doc = "However, an input that is active but not changing, will re-transmit the last valid ArtDmx packet at approximately 4-second intervals. (Note. In order to converge the needs of ArtNet and sACN it is recommended that Art-Net devices actually use a re-transmit time of 800mS to 1000mS)."]
    #[doc = ""]
    #[doc = "A DMX input that fails will not continue to transmit ArtDmx data."]
    pub struct Output {
        #[doc = "Determines which version the server has. Will be ARTNET_PROTOCOL_VERSION by default"]
        pub version: [u8; 2],
        #[doc = "The sequence number is used to ensure that ArtDmx packets are used in the correct order. When Art-Net is carried over a medium such as the Internet, it is possible that ArtDmx packets will reach the receiver out of order. This field is incremented in the range 0x01 to 0xff to allow the receiving node to resequence packets."]
        #[doc = ""]
        #[doc = "The Sequence field is set to 0x00 to disable this feature"]
        pub sequence: u8,
        #[doc = "The physical input port from which DMX512 data was input. This field is for information only. Use Universe for data routing"]
        pub physical: u8,
        #[doc = "The 15 bit Port-Address to which this packet is destined"]
        pub port_address: PortAddress,
        #[doc = "The length of the message, set by the artnet library itself"]
        pub length: BigEndianLength<Output>,
        #[doc = "A variable length array of DMX512 lighting data"]
        pub data: PaddedData,
    }
}

impl Default for Output {
    fn default() -> Output {
        Output {
            version: ARTNET_PROTOCOL_VERSION,
            sequence: 0,
            physical: 0,
            port_address: 1.into(),
            length: BigEndianLength::default(),
            data: PaddedData::default(),
        }
    }
}

#[derive(Default)]
pub struct PaddedData {
    inner: Vec<u8>,
}

impl PaddedData {
    fn len(&self) -> usize {
        self.inner.len()
    }
    fn len_rounded_up(&self) -> usize {
        let mut len = self.inner.len();
        if len % 2 != 0 {
            len += 1;
        }
        len
    }
}

impl From<Vec<u8>> for PaddedData {
    fn from(inner: Vec<u8>) -> Self {
        Self { inner }
    }
}

impl std::fmt::Debug for PaddedData {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{:?}", self.inner)
    }
}

impl<T> Convertable<T> for PaddedData {
    fn from_cursor(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        let remaining = cursor.get_ref();
        let inner = remaining[cursor.position() as usize..].to_vec();
        Ok(Self { inner })
    }

    fn into_buffer(&self, buffer: &mut Vec<u8>, _: &T) -> Result<()> {
        let len = self.len();
        if len == 0 {
            // packets must be between 2 and 512 bytes, 1 gets padded up, but 0 is invalid
            return Err(Error::MessageSizeInvalid {
                message: vec![],
                allowed_size: 2..512,
            });
        }
        if len > 512 {
            // packets must be between 2 and 512 bytes
            let inner = self.inner.clone();
            return Err(Error::MessageSizeInvalid {
                message: inner,
                allowed_size: 2..512,
            });
        }

        buffer.extend_from_slice(&self.inner[..]);
        if len % 2 != 0 {
            // the data of an output needs to be an even size, so we add an additional 0-byte
            buffer.push(0);
        }
        Ok(())
    }
    fn get_test_value() -> Self {
        PaddedData {
            inner: vec![1, 2, 3, 4],
        }
    }
    fn is_equal(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[derive(Default)]
pub struct BigEndianLength<T> {
    parsed_length: Option<u16>,
    _pd: std::marker::PhantomData<T>,
}

impl<T> std::fmt::Debug for BigEndianLength<T> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(len) = &self.parsed_length {
            write!(fmt, "{}", len)
        } else {
            write!(fmt, "Unknown (set during parsing)")
        }
    }
}

impl<T> std::ops::Deref for BigEndianLength<T> {
    type Target = u16;

    fn deref(&self) -> &u16 {
        self.parsed_length.as_ref().unwrap_or(&0)
    }
}

impl Convertable<Output> for BigEndianLength<Output> {
    fn from_cursor(cursor: &mut std::io::Cursor<&[u8]>) -> crate::Result<Self> {
        let length = cursor.read_u16::<BigEndian>().map_err(Error::CursorEof)?;
        Ok(BigEndianLength {
            parsed_length: Some(length),
            _pd: std::marker::PhantomData,
        })
    }
    fn into_buffer(&self, buffer: &mut Vec<u8>, context: &Output) -> crate::Result<()> {
        let len = context.data.len_rounded_up() as u16;
        buffer.write_u16::<BigEndian>(len).map_err(Error::CursorEof)
    }
    fn get_test_value() -> Self {
        Default::default()
    }
    fn is_equal(&self, other: &Self) -> bool {
        if (self.parsed_length.is_none() && other.parsed_length.is_some())
            || (self.parsed_length.is_some() && other.parsed_length.is_none())
        {
            // one of the two is parsed, but the other one isn't
            // They are not strictly equal, but we're testing for equality-after-parsing
            // and we don't know the length beforehand
            true
        } else {
            self.parsed_length == other.parsed_length
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArtCommand;

    mod serialization {
        use super::*;
        #[test]
        fn create_single_dmx_value_art_dmx_packet() {
            let command = ArtCommand::Output(Output {
                data: vec![255].into(), // The data we're sending to the node
                ..Output::default()
            });
            let bytes = command.into_buffer().unwrap();
            let comparison = vec![
                65, 114, 116, 45, 78, 101, 116, 0, 0, 80, 0, 14, 0, 0, 1, 0, 0, 2, 255, 0,
            ]; //is padded with zero to even length of two
            assert_eq!(bytes, comparison)
        }
        #[test]
        fn create_512_dmx_values_art_dmx_packet() {
            let command = ArtCommand::Output(Output {
                data: vec![128; 512].into(), // The data we're sending to the node
                ..Output::default()
            });
            let bytes = command.into_buffer().unwrap();
            let comparison = vec![
                vec![
                    65, 114, 116, 45, 78, 101, 116, 0, 0, 80, 0, 14, 0, 0, 1, 0, 2, 0,
                ],
                vec![128; 512],
            ]
            .concat(); //is padded with zero to even length of two
            assert_eq!(bytes, comparison)
        }
        #[test]
        fn test_invalid_length() {
            let command = ArtCommand::Output(Output {
                data: vec![0xff; 512].into(),
                ..Output::default()
            });
            let buffer = command.into_buffer().unwrap();
            // #6: length needs to be encoded in big endian
            assert_eq!(&buffer[0x10..=0x11], &[2, 0]);
            // #7.1: packets need to be an even number
            fn get_data(command: &ArtCommand) -> &PaddedData {
                if let ArtCommand::Output(output) = command {
                    &output.data
                } else {
                    unreachable!()
                }
            };
            let command = ArtCommand::Output(Output {
                data: vec![0xff].into(),
                ..Output::default()
            });
            // Initially it will be 1
            assert_eq!(get_data(&command).len(), 1);
            // But the padded length is 2
            assert_eq!(get_data(&command).len_rounded_up(), 2);
            let buffer = command.into_buffer().unwrap();
            // The data written is 2 bytes
            assert_eq!(&buffer[0x10..=0x11], &[0, 2]);
            // #7.2: packets need to be at least 2 bytes
            let command = ArtCommand::Output(Output {
                data: vec![].into(),
                ..Output::default()
            });
            assert!(command.into_buffer().is_err());
            // #7.3: packets need to be at most 512 bytes
            let command = ArtCommand::Output(Output {
                data: vec![0xff; 513].into(),
                ..Output::default()
            });
            assert!(command.into_buffer().is_err());
        }
    }

    mod parsing {
        use super::*;

        #[test]
        fn protver_below_14() {
            // Because Art-Net is guaranteed to be backwards-compatible,
            // we should be able to parse versions below 14,
            // even tough these should never be seen in the wild
            let packet = &vec![
                65, 114, 116, 45, 78, 101, 116, 0, 0, 80, 0, 0, 0, 0, 1, 0, 0, 2, 255, 255,
            ];
            let command = ArtCommand::from_buffer(packet).unwrap();
            if let ArtCommand::Output(output) = command {
                assert_eq!(output.version, [0, 0]);
                assert_eq!(output.sequence, 0);
                assert_eq!(output.physical, 0);
                assert_eq!(output.port_address, 1.into());
                assert_eq!(output.length.parsed_length, Some(2));
                assert_eq!(output.data.inner, vec![255, 255]);
            }
        }

        #[test]
        fn invalid_port_address() {
            // Here Port-Address is 32_768
            // Any Port-Address over 32_767 should fail
            assert!(ArtCommand::from_buffer(
                &vec![
                    vec![65, 114, 116, 45, 78, 101, 116, 0, 0, 80, 0, 14, 0, 0,],
                    32_768u16.to_le_bytes().to_vec(),
                    vec![0, 2, 255, 255,],
                ]
                .concat()
            )
            .is_err());
        }
    }
}
