//! A TZif parser for versions 1 to 3.

use std::{
    io::{self, Read, Seek, SeekFrom},
    str::FromStr,
};

use crate::{
    error::ParseError,
    posix::PosixTimeZone,
    timestamp::NaiveTimestamp,
    transitions::{Transition, TransitionType, Transitions},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Header {
    /// The version of the TZif file.
    version: u8,
    /// The number of UTC/local indicators.
    ///
    /// Corresponds to `isutcnt` in RFC 8536.
    utc_locals_count: u32,
    /// The number of standard/wall indicators.
    ///
    /// Corresponds to `isstdcnt` in RFC 8536.
    std_count: u32,
    /// The number of leap seconds.
    ///
    /// Corresponds to `leapcnt` in RFC 8536.
    leaps: u32,
    /// The number of transitions.
    ///
    /// Corresponds to `timecnt` in RFC 8536.
    transitions: u32,
    /// The number of "local time types". This number cannot be zero.
    ///
    /// Corresponds to `typecnt` in RFC 8536.
    types: u32,
    /// Corresponds to `charcnt` in RFC 8536.
    abbr_size: u32,
}

// TODO: Remove once array_chunks is stabilised!
fn array_chunks<'a, T, const N: usize>(slice: &'a [T]) -> std::slice::Iter<'a, [T; N]> {
    let len = slice.len() / N;
    let (chunks, _) = slice.split_at(len * N);
    // SAFETY: Conversion from len * N elements into chunks of N elements is provably safe
    // this code comes from the stdlib!
    let chunks: &'a [[T; N]] = unsafe { std::slice::from_raw_parts(chunks.as_ptr().cast(), len) };
    chunks.iter()
}

impl Header {
    fn version_one_length(&self) -> i64 {
        self.transitions as i64 * 5
            + self.types as i64 * 6
            + self.abbr_size as i64
            + self.leaps as i64 * 8
            + self.std_count as i64
            + self.utc_locals_count as i64
    }

    fn from_reader<R: Read>(reader: &mut R) -> Result<Self, ParseError> {
        // 7 * 4 + 1 + 15 = 44
        let mut buffer = [0u8; 44];
        reader.read_exact(&mut buffer)?;
        if &buffer[0..4] != b"TZif" {
            return Err(ParseError::InvalidMagic);
        }

        let version = match buffer[4] {
            0 => 1,
            b'2' => 2,
            b'3' => 3,
            _ => return Err(ParseError::UnsupportedVersion),
        };

        // Skip 15 bytes (so index 5 -> 19 are irrelevant)

        Ok(Self {
            version,
            utc_locals_count: u32::from_be_bytes([buffer[20], buffer[21], buffer[22], buffer[23]]),
            std_count: u32::from_be_bytes([buffer[24], buffer[25], buffer[26], buffer[27]]),
            leaps: u32::from_be_bytes([buffer[28], buffer[29], buffer[30], buffer[31]]),
            transitions: u32::from_be_bytes([buffer[32], buffer[33], buffer[34], buffer[35]]),
            types: u32::from_be_bytes([buffer[36], buffer[37], buffer[38], buffer[39]]),
            abbr_size: u32::from_be_bytes([buffer[40], buffer[41], buffer[42], buffer[43]]),
        })
    }

    fn read_transitions<R: Read>(&self, reader: &mut R) -> Result<Vec<i64>, io::Error> {
        if self.transitions != 0 {
            let mut buffer = vec![0u8; self.transitions as usize * 8];
            reader.read_exact(&mut buffer)?;
            Ok(array_chunks::<u8, 8>(&buffer).map(|x| i64::from_be_bytes(*x)).collect())
        } else {
            Ok(vec![])
        }
    }

    fn read_transitions_32<R: Read>(&self, reader: &mut R) -> Result<Vec<i64>, io::Error> {
        if self.transitions != 0 {
            let mut buffer = vec![0u8; self.transitions as usize * 4];
            reader.read_exact(&mut buffer)?;
            Ok(array_chunks::<u8, 4>(&buffer)
                .map(|x| i32::from_be_bytes(*x) as i64)
                .collect())
        } else {
            Ok(vec![])
        }
    }

    fn read_transition_indexes<R: Read>(&self, reader: &mut R) -> Result<Vec<u8>, io::Error> {
        if self.transitions != 0 {
            let mut indexes = vec![0u8; self.transitions as usize];
            reader.read_exact(&mut indexes)?;
            Ok(indexes)
        } else {
            Ok(vec![])
        }
    }

    fn read_transition_types<R: Read>(&self, reader: &mut R) -> Result<Vec<TransitionType>, ParseError> {
        if self.types != 0 {
            // First, read a buffer for all the ttypes necessary. Each of these are
            // 6 bytes each and should be easy to fill into a pre-allocated buffer.
            // This requires two passes because eventually we need to read the
            // abbreviation block in order to feed it into the TransitionType type.
            let mut buffer = vec![0u8; self.types as usize * 6];
            reader.read_exact(&mut buffer)?;

            // At this point we can read a byte buffer with the C-string names for timezones
            // For some reason this information is specified as an array of null terminated strings
            // The ttypes data type above uses an index to index into this strings buffer up until
            // it finds a null terminator. I suppose this makes sense in C where you have many APIs
            // that work up until a null terminator.
            let mut strings = vec![0u8; self.abbr_size as usize];
            reader.read_exact(&mut strings)?;

            // Now that all the I/O is done, we can actually convert the read data into transition types.
            // The transition types buffer is an array of utc offset in seconds (i32), dst (u8), and idx (u8).
            // The idx is used to index into the strings buffer up until a null terminator is found.
            let mut transitions = Vec::with_capacity(self.types as usize);
            for &[seconds @ .., dst, idx] in array_chunks::<u8, 6>(&buffer) {
                let offset = i32::from_be_bytes(seconds);
                let idx = idx as usize;
                let abbr = {
                    if let Some(index) = strings.iter().skip(idx).position(|&c| c == 0) {
                        let end = idx + index;
                        let s = std::str::from_utf8(&strings[idx..end]).map_err(|_| ParseError::InvalidAbbreviation)?;
                        String::from(s)
                    } else {
                        String::new()
                    }
                };

                transitions.push(TransitionType {
                    offset,
                    is_dst: dst == 1,
                    abbr,
                });
            }
            Ok(transitions)
        } else {
            Ok(vec![])
        }
    }

    fn get_transitions<R: Read + Seek>(&self, reader: &mut R) -> Result<Transitions, ParseError> {
        let trans = if self.version == 1 {
            self.read_transitions_32(reader)?
        } else {
            self.read_transitions(reader)?
        };
        let idxs = self.read_transition_indexes(reader)?;
        let ttypes = self.read_transition_types(reader)?;

        // TODO: leap seconds?
        reader.seek(SeekFrom::Current(
            self.leaps as i64 * 12 + self.std_count as i64 + self.utc_locals_count as i64,
        ))?;

        let posix = if self.version >= 2 {
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer)?;
            let s = String::from_utf8(buffer).map_err(|_| ParseError::InvalidPosixTz)?;
            if s.starts_with('\n') && s.ends_with('\n') {
                let s = s.as_str();
                Some(PosixTimeZone::from_str(&s[1..s.len() - 1])?)
            } else {
                return Err(ParseError::InvalidPosixTz);
            }
        } else {
            None
        };

        let mut transitions: Vec<Transition> = Vec::with_capacity(trans.len());
        for (trans, idx) in trans.iter().zip(idxs.iter()) {
            let prev = transitions.last().map(|p| ttypes.get(p.type_idx)).flatten();
            let idx = *idx as usize;
            let ttype = &ttypes[idx];
            let transition = Transition::new(NaiveTimestamp(*trans), idx, ttype, prev);
            transitions.push(transition);
        }

        Ok(Transitions {
            data: transitions,
            types: ttypes,
            posix,
        })
    }
}

pub(crate) fn parse_tzif<R: Read + Seek>(mut reader: R) -> Result<Transitions, ParseError> {
    let mut header = Header::from_reader(&mut reader)?;
    if header.version == 1 {
        header.get_transitions(&mut reader)
    } else {
        reader.seek(SeekFrom::Current(header.version_one_length()))?;
        header = Header::from_reader(&mut reader)?;
        header.get_transitions(&mut reader)
    }
}
