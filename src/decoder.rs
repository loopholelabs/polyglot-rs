/*
    Copyright 2022 Loophole Labs

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

           http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

use crate::kind::Kind;
use byteorder::{BigEndian, ReadBytesExt};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{Cursor, Read};
use std::str;

#[derive(Debug, PartialEq)]
pub enum DecodingError {
    InvalidNone,
    InvalidArray,
    InvalidMap,
    InvalidBytes,
    InvalidString,
    InvalidError,
    InvalidBool,
    InvalidU8,
    InvalidU16,
    InvalidU32,
    InvalidU64,
    InvalidI32,
    InvalidI64,
    InvalidF32,
    InvalidF64,
    InvalidEnum,
    InvalidStruct,
}

impl Display for DecodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for DecodingError {}

const VARINT_LEN16: u16 = 3;
const VARINT_LEN32: u32 = 5;
const VARINT_LEN64: u64 = 10;
const CONTINUATION: u8 = 0x80;

pub trait Decoder {
    fn decode_none(&mut self) -> bool;
    fn decode_array(&mut self, val_kind: Kind) -> Result<usize, DecodingError>;
    fn decode_map(&mut self, key_kind: Kind, val_kind: Kind) -> Result<usize, DecodingError>;
    fn decode_bytes(&mut self) -> Result<Vec<u8>, DecodingError>;
    fn decode_string(&mut self) -> Result<String, DecodingError>;
    fn decode_error(&mut self) -> Result<Box<dyn Error>, DecodingError>;
    fn decode_bool(&mut self) -> Result<bool, DecodingError>;
    fn decode_u8(&mut self) -> Result<u8, DecodingError>;
    fn decode_u16(&mut self) -> Result<u16, DecodingError>;
    fn decode_u32(&mut self) -> Result<u32, DecodingError>;
    fn decode_u64(&mut self) -> Result<u64, DecodingError>;
    fn decode_i32(&mut self) -> Result<i32, DecodingError>;
    fn decode_i64(&mut self) -> Result<i64, DecodingError>;
    fn decode_f32(&mut self) -> Result<f32, DecodingError>;
    fn decode_f64(&mut self) -> Result<f64, DecodingError>;
}

impl Decoder for Cursor<&mut Vec<u8>> {
    fn decode_none(&mut self) -> bool {
        if let Ok(kind) = self.read_u8() {
            if kind == Kind::None as u8 {
                return true;
            }
        }
        self.set_position(self.position() - 1);
        false
    }

    fn decode_array(&mut self, val_kind: Kind) -> Result<usize, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidArray)?;
        let defined_val_kind = self.read_u8().ok().ok_or(DecodingError::InvalidArray)?;
        if kind == Kind::Array as u8 && val_kind as u8 == defined_val_kind {
            return match self.decode_u32() {
                Err(err) => Err(err),
                Ok(val) => Ok(val as usize),
            };
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidU32)
    }

    fn decode_map(&mut self, key_kind: Kind, val_kind: Kind) -> Result<usize, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidMap)?;
        let defined_key_kind = self.read_u8().ok().ok_or(DecodingError::InvalidMap)?;
        let defined_val_kind = self.read_u8().ok().ok_or(DecodingError::InvalidMap)?;
        if kind == Kind::Map as u8
            && key_kind as u8 == defined_key_kind
            && val_kind as u8 == defined_val_kind
        {
            return match self.decode_u32() {
                Err(err) => Err(err),
                Ok(val) => Ok(val as usize),
            };
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidMap)
    }

    fn decode_bytes(&mut self) -> Result<Vec<u8>, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidBytes)?;
        if kind == Kind::Bytes as u8 {
            let size = self.decode_u32()? as usize;
            let mut buf = vec![0u8; size];
            self.read_exact(&mut buf)
                .ok()
                .ok_or(DecodingError::InvalidBytes)?;
            return Ok(buf);
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidBytes)
    }

    fn decode_string(&mut self) -> Result<String, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidString)?;

        if kind == Kind::String as u8 {
            let size = self.decode_u32()? as usize;
            let mut str_buf = vec![0u8; size];
            self.read_exact(&mut str_buf)
                .ok()
                .ok_or(DecodingError::InvalidString)?;

            let result = str::from_utf8(&str_buf)
                .ok()
                .ok_or(DecodingError::InvalidString)?;
            return Ok(result.to_owned());
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidString)
    }

    fn decode_error(&mut self) -> Result<Box<dyn Error>, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidError)?;
        let nested_kind = self.read_u8().ok().ok_or(DecodingError::InvalidError)?;
        if kind == Kind::Error as u8 && nested_kind == Kind::String as u8 {
            let size = self.decode_u32()? as usize;
            let mut str_buf = vec![0u8; size];
            self.read_exact(&mut str_buf)
                .ok()
                .ok_or(DecodingError::InvalidError)?;

            let result = str::from_utf8(&str_buf)
                .ok()
                .ok_or(DecodingError::InvalidError)?;
            return Ok(Box::<dyn Error>::from(result.to_owned()));
        }
        self.set_position(self.position() - 2);
        Err(DecodingError::InvalidError)
    }

    fn decode_bool(&mut self) -> Result<bool, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidBool)?;
        if kind == Kind::Bool as u8 {
            let val = self.read_u8().ok().ok_or(DecodingError::InvalidBool)?;
            return Ok(val == 1);
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidBool)
    }

    fn decode_u8(&mut self) -> Result<u8, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidU8)?;
        if kind == Kind::U8 as u8 {
            return self.read_u8().ok().ok_or(DecodingError::InvalidU8);
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidU8)
    }

    fn decode_u16(&mut self) -> Result<u16, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidU16)?;
        if kind == Kind::U16 as u8 {
            let mut x: u16 = 0;
            let mut s: u32 = 0;

            for _ in 0..VARINT_LEN16 {
                let byte = self.read_u8().ok().ok_or(DecodingError::InvalidU16)?;
                if byte < CONTINUATION {
                    return Ok(x | (byte as u16) << s);
                }
                x |= (byte as u16 & ((CONTINUATION as u16) - 1)) << s;
                s += 7;
            }
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidU32)
    }

    fn decode_u32(&mut self) -> Result<u32, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidU32)?;
        if kind == Kind::U32 as u8 {
            let mut x: u32 = 0;
            let mut s: u32 = 0;

            for _ in 0..VARINT_LEN32 {
                let byte = self.read_u8().ok().ok_or(DecodingError::InvalidU32)?;
                if byte < CONTINUATION {
                    return Ok(x | (byte as u32) << s);
                }
                x |= (byte as u32 & ((CONTINUATION as u32) - 1)) << s;
                s += 7;
            }
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidU32)
    }

    fn decode_u64(&mut self) -> Result<u64, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidU64)?;
        if kind == Kind::U64 as u8 {
            let mut x: u64 = 0;
            let mut s: u32 = 0;

            for _ in 0..VARINT_LEN64 {
                let byte = self.read_u8().ok().ok_or(DecodingError::InvalidU64)?;
                if byte < CONTINUATION {
                    return Ok(x | (byte as u64) << s);
                }
                x |= (byte as u64 & ((CONTINUATION as u64) - 1)) << s;
                s += 7;
            }
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidU32)
    }

    fn decode_i32(&mut self) -> Result<i32, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidI32)?;
        if kind == Kind::I32 as u8 {
            let mut ux: u32 = 0;
            let mut s: u32 = 0;

            for _ in 0..VARINT_LEN32 {
                let byte = self.read_u8().ok().ok_or(DecodingError::InvalidI32)?;
                if byte < CONTINUATION {
                    let mut x = ((ux | (byte as u32) << s) >> 1) as i32;
                    if ux & 1 != 0 {
                        x = !x
                    }
                    return Ok(x);
                }
                ux |= (byte as u32 & ((CONTINUATION as u32) - 1)) << s;
                s += 7;
            }
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidI32)
    }

    fn decode_i64(&mut self) -> Result<i64, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidI64)?;
        if kind == Kind::I64 as u8 {
            let mut ux: u64 = 0;
            let mut s: u32 = 0;

            for _ in 0..VARINT_LEN64 {
                let byte = self.read_u8().ok().ok_or(DecodingError::InvalidI64)?;
                if byte < CONTINUATION {
                    let mut x = ((ux | (byte as u64) << s) >> 1) as i64;
                    if ux & 1 != 0 {
                        x = !x
                    }
                    return Ok(x);
                }
                ux |= (byte as u64 & ((CONTINUATION as u64) - 1)) << s;
                s += 7;
            }
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidI64)
    }

    fn decode_f32(&mut self) -> Result<f32, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidF32)?;
        if kind == Kind::F32 as u8 {
            return self
                .read_f32::<BigEndian>()
                .ok()
                .ok_or(DecodingError::InvalidF32);
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidF32)
    }

    fn decode_f64(&mut self) -> Result<f64, DecodingError> {
        let kind = self.read_u8().ok().ok_or(DecodingError::InvalidF64)?;
        if kind == Kind::F64 as u8 {
            return self
                .read_f64::<BigEndian>()
                .ok()
                .ok_or(DecodingError::InvalidF64);
        }
        self.set_position(self.position() - 1);
        Err(DecodingError::InvalidF64)
    }
}
