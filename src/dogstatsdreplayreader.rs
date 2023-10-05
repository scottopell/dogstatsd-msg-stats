use std::collections::VecDeque;
use thiserror::Error;

use bytes::Bytes;

use crate::replay::{ReplayReader, ReplayReaderError};

pub mod dogstatsd {
    pub mod unix {
        include!(concat!(env!("OUT_DIR"), "/dogstatsd.unix.rs"));
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum DogStatsDReplayReaderError {
    #[error("No dogstatsd replay marker found")]
    NotAReplayFile,
    #[error("Unsupported replay version")]
    UnsupportedReplayVersion,
    #[error("Invalid UTF-8 sequence found in payload of msg")]
    InvalidUtf8Sequence,
}

pub struct DogStatsDReplayReader {
    replay_msg_reader: ReplayReader,
    current_messages: VecDeque<String>,
}

impl DogStatsDReplayReader {
    pub fn read_msg(&mut self, s: &mut String) -> Result<usize, DogStatsDReplayReaderError> {
        if let Some(line) = self.current_messages.pop_front() {
            s.insert_str(0, &line);
            return Ok(1);
        }

        match self.replay_msg_reader.read_msg() {
            Some(msg) => {
                match std::str::from_utf8(&msg.payload) {
                    Ok(v) => {
                        if v.is_empty() {
                            // Read operation was successful, read 0 msgs
                            return Ok(0);
                        }

                        for line in v.lines() {
                            self.current_messages.push_back(String::from(line));
                        }

                        self.read_msg(s)
                    }
                    Err(e) => Err(DogStatsDReplayReaderError::InvalidUtf8Sequence), // TODO add the msg or msg.payload that has the issue
                }
            }
            None => Ok(0), // Read was validly issued, just nothing to be read.
        }
    }

    pub fn new(mut buf: Bytes) -> Result<Self, DogStatsDReplayReaderError> {
        match ReplayReader::new(buf) {
            Ok(reader) => Ok(DogStatsDReplayReader {
                replay_msg_reader: reader,
                current_messages: VecDeque::new(),
            }),
            Err(e) => match e {
                ReplayReaderError::NotAReplayFile => {
                    Err(DogStatsDReplayReaderError::NotAReplayFile)
                }
                ReplayReaderError::UnsupportedReplayVersion => {
                    Err(DogStatsDReplayReaderError::UnsupportedReplayVersion)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_MSGS_ONE_LINE_EACH: &[u8] = &[
        0xd4, 0x74, 0xd0, 0x60, 0xf3, 0xff, 0x00, 0x00, 0x93, 0x00, 0x00, 0x00, 0x08, 0x84, 0xe2,
        0x88, 0x8a, 0xe0, 0xb6, 0x87, 0xbf, 0x17, 0x10, 0x83, 0x01, 0x1a, 0x83, 0x01, 0x73, 0x74,
        0x61, 0x74, 0x73, 0x64, 0x2e, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x2e, 0x74, 0x69,
        0x6d, 0x65, 0x2e, 0x6d, 0x69, 0x63, 0x72, 0x6f, 0x73, 0x3a, 0x32, 0x2e, 0x33, 0x39, 0x32,
        0x38, 0x33, 0x7c, 0x64, 0x7c, 0x40, 0x31, 0x2e, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x7c,
        0x23, 0x65, 0x6e, 0x76, 0x69, 0x72, 0x6f, 0x6e, 0x6d, 0x65, 0x6e, 0x74, 0x3a, 0x64, 0x65,
        0x76, 0x7c, 0x63, 0x3a, 0x32, 0x61, 0x32, 0x35, 0x66, 0x37, 0x66, 0x63, 0x38, 0x66, 0x62,
        0x66, 0x35, 0x37, 0x33, 0x64, 0x36, 0x32, 0x30, 0x35, 0x33, 0x64, 0x37, 0x32, 0x36, 0x33,
        0x64, 0x64, 0x32, 0x64, 0x34, 0x34, 0x30, 0x63, 0x30, 0x37, 0x62, 0x36, 0x61, 0x62, 0x34,
        0x64, 0x32, 0x62, 0x31, 0x30, 0x37, 0x65, 0x35, 0x30, 0x62, 0x30, 0x64, 0x34, 0x64, 0x66,
        0x31, 0x66, 0x32, 0x65, 0x65, 0x31, 0x35, 0x66, 0x0a, 0x93, 0x00, 0x00, 0x00, 0x08, 0x9f,
        0xe9, 0xbd, 0x83, 0xe3, 0xb6, 0x87, 0xbf, 0x17, 0x10, 0x83, 0x01, 0x1a, 0x83, 0x01, 0x73,
        0x74, 0x61, 0x74, 0x73, 0x64, 0x2e, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x2e, 0x74,
        0x69, 0x6d, 0x65, 0x2e, 0x6d, 0x69, 0x63, 0x72, 0x6f, 0x73, 0x3a, 0x32, 0x2e, 0x33, 0x39,
        0x32, 0x38, 0x33, 0x7c, 0x64, 0x7c, 0x40, 0x31, 0x2e, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x7c, 0x23, 0x65, 0x6e, 0x76, 0x69, 0x72, 0x6f, 0x6e, 0x6d, 0x65, 0x6e, 0x74, 0x3a, 0x64,
        0x65, 0x76, 0x7c, 0x63, 0x3a, 0x32, 0x61, 0x32, 0x35, 0x66, 0x37, 0x66, 0x63, 0x38, 0x66,
        0x62, 0x66, 0x35, 0x37, 0x33, 0x64, 0x36, 0x32, 0x30, 0x35, 0x33, 0x64, 0x37, 0x32, 0x36,
        0x33, 0x64, 0x64, 0x32, 0x64, 0x34, 0x34, 0x30, 0x63, 0x30, 0x37, 0x62, 0x36, 0x61, 0x62,
        0x34, 0x64, 0x32, 0x62, 0x31, 0x30, 0x37, 0x65, 0x35, 0x30, 0x62, 0x30, 0x64, 0x34, 0x64,
        0x66, 0x31, 0x66, 0x32, 0x65, 0x65, 0x31, 0x35, 0x66, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00,
    ];

    const ONE_MSG_TWO_LINES: &[u8] = &[
        0xd4, 0x74, 0xd0, 0x60, 0xf3, 0xff, 0x00, 0x00, 0xe6, 0x00, 0x00, 0x00, 0x08, 0xf7, 0xc3,
        0xb4, 0xdc, 0xfa, 0x85, 0x88, 0xbf, 0x17, 0x10, 0xd6, 0x01, 0x1a, 0xd6, 0x01, 0x73, 0x74,
        0x61, 0x74, 0x73, 0x64, 0x2e, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x2e, 0x74, 0x69,
        0x6d, 0x65, 0x2e, 0x6d, 0x69, 0x63, 0x72, 0x6f, 0x73, 0x3a, 0x32, 0x2e, 0x33, 0x39, 0x32,
        0x38, 0x33, 0x7c, 0x64, 0x7c, 0x40, 0x31, 0x2e, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x7c,
        0x23, 0x65, 0x6e, 0x76, 0x69, 0x72, 0x6f, 0x6e, 0x6d, 0x65, 0x6e, 0x74, 0x3a, 0x64, 0x65,
        0x76, 0x2c, 0x6e, 0x6f, 0x77, 0x3a, 0x32, 0x30, 0x32, 0x33, 0x2d, 0x30, 0x38, 0x2d, 0x32,
        0x33, 0x54, 0x32, 0x31, 0x3a, 0x32, 0x34, 0x3a, 0x35, 0x39, 0x2b, 0x30, 0x30, 0x3a, 0x30,
        0x30, 0x7c, 0x63, 0x3a, 0x32, 0x61, 0x32, 0x35, 0x66, 0x37, 0x66, 0x63, 0x38, 0x66, 0x62,
        0x66, 0x35, 0x37, 0x33, 0x64, 0x36, 0x32, 0x30, 0x35, 0x33, 0x64, 0x37, 0x32, 0x36, 0x33,
        0x64, 0x64, 0x32, 0x64, 0x34, 0x34, 0x30, 0x63, 0x30, 0x37, 0x62, 0x36, 0x61, 0x62, 0x34,
        0x64, 0x32, 0x62, 0x31, 0x30, 0x37, 0x65, 0x35, 0x30, 0x62, 0x30, 0x64, 0x34, 0x64, 0x66,
        0x31, 0x66, 0x32, 0x65, 0x65, 0x31, 0x35, 0x66, 0x0a, 0x73, 0x74, 0x61, 0x74, 0x73, 0x64,
        0x2e, 0x6f, 0x74, 0x68, 0x65, 0x72, 0x2e, 0x6d, 0x65, 0x74, 0x72, 0x69, 0x63, 0x3a, 0x38,
        0x2e, 0x37, 0x7c, 0x67, 0x7c, 0x40, 0x31, 0x2e, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x7c,
        0x23, 0x65, 0x6e, 0x76, 0x69, 0x72, 0x6f, 0x6e, 0x6d, 0x65, 0x6e, 0x74, 0x3a, 0x64, 0x65,
        0x76, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    const ONE_MSG_THREE_LINES: &[u8] = &[
        0xd4, 0x74, 0xd0, 0x60, 0xf3, 0xff, 0x00, 0x00, 0xa9, 0x00, 0x00, 0x00, 0x08, 0xa7, 0xe3,
        0x97, 0xff, 0xaf, 0xbb, 0x88, 0xbf, 0x17, 0x10, 0x99, 0x01, 0x1a, 0x99, 0x01, 0x73, 0x74,
        0x61, 0x74, 0x73, 0x64, 0x2e, 0x6f, 0x74, 0x68, 0x65, 0x72, 0x2e, 0x6d, 0x65, 0x74, 0x72,
        0x69, 0x63, 0x3a, 0x33, 0x7c, 0x63, 0x7c, 0x40, 0x31, 0x2e, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x7c, 0x23, 0x65, 0x6e, 0x76, 0x69, 0x72, 0x6f, 0x6e, 0x6d, 0x65, 0x6e, 0x74, 0x3a,
        0x64, 0x65, 0x76, 0x0a, 0x73, 0x74, 0x61, 0x74, 0x73, 0x64, 0x2e, 0x6f, 0x74, 0x68, 0x65,
        0x72, 0x2e, 0x6d, 0x65, 0x74, 0x72, 0x69, 0x63, 0x3a, 0x38, 0x7c, 0x63, 0x7c, 0x40, 0x31,
        0x2e, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x7c, 0x23, 0x65, 0x6e, 0x76, 0x69, 0x72, 0x6f,
        0x6e, 0x6d, 0x65, 0x6e, 0x74, 0x3a, 0x64, 0x65, 0x76, 0x0a, 0x73, 0x74, 0x61, 0x74, 0x73,
        0x64, 0x2e, 0x6f, 0x74, 0x68, 0x65, 0x72, 0x2e, 0x6d, 0x65, 0x74, 0x72, 0x69, 0x63, 0x3a,
        0x37, 0x7c, 0x63, 0x7c, 0x40, 0x31, 0x2e, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x7c, 0x23,
        0x65, 0x6e, 0x76, 0x69, 0x72, 0x6f, 0x6e, 0x6d, 0x65, 0x6e, 0x74, 0x3a, 0x64, 0x65, 0x76,
        0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[test]
    fn two_msg_two_lines() {
        let mut replay = DogStatsDReplayReader::new(Bytes::from(TWO_MSGS_ONE_LINE_EACH)).unwrap();
        let mut s = String::new();
        let res = replay.read_msg(&mut s).unwrap();
        assert_eq!(res, 1);
        assert_eq!("statsd.example.time.micros:2.39283|d|@1.000000|#environment:dev|c:2a25f7fc8fbf573d62053d7263dd2d440c07b6ab4d2b107e50b0d4df1f2ee15f", s);
        s.clear();
        let res = replay.read_msg(&mut s).unwrap();
        assert_eq!(res, 1);
        assert_eq!("statsd.example.time.micros:2.39283|d|@1.000000|#environment:dev|c:2a25f7fc8fbf573d62053d7263dd2d440c07b6ab4d2b107e50b0d4df1f2ee15f", s);
        let res = replay.read_msg(&mut s).unwrap();
        assert_eq!(res, 0);
    }

    #[test]
    fn one_msg_two_lines() {
        let mut replay = DogStatsDReplayReader::new(Bytes::from(ONE_MSG_TWO_LINES)).unwrap();
        let mut s = String::new();
        let res = replay.read_msg(&mut s).unwrap();
        assert_eq!(res, 1);
        assert_eq!("statsd.example.time.micros:2.39283|d|@1.000000|#environment:dev,now:2023-08-23T21:24:59+00:00|c:2a25f7fc8fbf573d62053d7263dd2d440c07b6ab4d2b107e50b0d4df1f2ee15f", s);
        s.clear();
        let res = replay.read_msg(&mut s).unwrap();
        assert_eq!(res, 1);
        assert_eq!("statsd.other.metric:8.7|g|@1.000000|#environment:dev", s);
        let res = replay.read_msg(&mut s).unwrap();
        assert_eq!(res, 0);
    }

    #[test]
    fn one_msg_three_lines() {
        let mut replay = DogStatsDReplayReader::new(Bytes::from(ONE_MSG_THREE_LINES)).unwrap();
        let mut s = String::new();

        let res = replay.read_msg(&mut s).unwrap();
        assert_eq!(res, 1);
        assert_eq!("statsd.other.metric:3|c|@1.000000|#environment:dev", s);
        s.clear();

        let res = replay.read_msg(&mut s).unwrap();
        assert_eq!(res, 1);
        assert_eq!("statsd.other.metric:8|c|@1.000000|#environment:dev", s);
        s.clear();

        let res = replay.read_msg(&mut s).unwrap();
        assert_eq!(res, 1);
        assert_eq!("statsd.other.metric:7|c|@1.000000|#environment:dev", s);
        s.clear();

        let res = replay.read_msg(&mut s).unwrap();
        assert_eq!(res, 0);
    }
}
