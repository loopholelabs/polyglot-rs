extern crate polyglot;

use std::io::Cursor;
use polyglot::Encoder;
use polyglot::Kind;

#[test]
fn test_encode_nil () {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    encoder = encoder.encode_none();

    assert_eq!(encoder.position(), 1);
    assert_eq!(encoder.get_ref()[0], Kind::None as u8);
}

#[test]
fn test_encode_array() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    encoder = encoder.encode_array(32, Kind::String);

    assert_eq!(encoder.position(), 1+1+1+4);
    assert_eq!(encoder.get_ref()[0], Kind::Array as u8);
    assert_eq!(encoder.get_ref()[1], Kind::String as u8);
    assert_eq!(encoder.get_ref()[2], Kind::U32 as u8);
}

#[test]
fn test_encode_map() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    encoder = encoder.encode_map(32, Kind::String, Kind::U32);

    assert_eq!(encoder.position(), 1+1+1+1+4);
    assert_eq!(encoder.get_ref()[0], Kind::Map as u8);
    assert_eq!(encoder.get_ref()[1], Kind::String as u8);
    assert_eq!(encoder.get_ref()[2], Kind::U32 as u8);
    assert_eq!(encoder.get_ref()[3], Kind::U32 as u8);
}

#[test]
fn test_encode_bytes() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    let v = "Test String".as_bytes();
    encoder = encoder.encode_bytes(v);

    assert_eq!(encoder.position() as usize, 1+1+4+v.len());
    assert_eq!(encoder.get_ref()[1+1+4..].to_owned(), v);
}

#[test]
fn test_encode_string() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    let v = "Test String";
    encoder = encoder.encode_string(v);

    assert_eq!(encoder.position() as usize, 1+1+1+4+v.len());
    assert_eq!(encoder.get_ref()[1+1+1+4..].to_owned(), v.as_bytes());
}

#[test]
fn test_encode_error() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    let v = "Test Error";
    encoder = encoder.encode_error(v);
    assert_eq!(encoder.position() as usize, 1+1+4+v.len());
    assert_eq!(encoder.get_ref()[1+1+4..].to_owned(), v.to_string().as_bytes());
}

#[test]
fn test_encode_bool() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    encoder = encoder.encode_bool(true);

    assert_eq!(encoder.position(), 2);
    assert_eq!(encoder.get_ref()[1], 0x1);
}

#[test]
fn test_encode_u8() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    encoder = encoder.encode_u8(32);

    assert_eq!(encoder.position(), 2);
    assert_eq!(encoder.get_ref()[1], 32);
}

#[test]
fn test_encode_u16() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    let v = 1024;
    let e = [0x4, 0x0];
    encoder = encoder.encode_u16(v);

    assert_eq!(encoder.position(), 3);
    assert_eq!(encoder.get_ref()[1..].to_owned(), e);
}

#[test]
fn test_encode_u32() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    let v = 4294967290;
    let e = [0xFF, 0xFF, 0xFF, 0xFA];
    encoder = encoder.encode_u32(v);

    assert_eq!(encoder.position(), 5);
    assert_eq!(encoder.get_ref()[1..].to_owned(), e);
}

#[test]
fn test_encode_u64() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    let v = 18446744073709551610;
    let e = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFA];
    encoder = encoder.encode_u64(v);

    assert_eq!(encoder.position(), 9);
    assert_eq!(encoder.get_ref()[1..].to_owned(), e);
}

#[test]
fn test_encode_i32() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    let v = -2147483648;
    let e = [0x80, 0x0, 0x0, 0x0];
    encoder = encoder.encode_i32(v);

    assert_eq!(encoder.position(), 5);
    assert_eq!(encoder.get_ref()[1..].to_owned(), e);
}

#[test]
fn test_encode_i64() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    let v = -9223372036854775808 as i64;
    let e = [0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];
    encoder = encoder.encode_i64(v);

    assert_eq!(encoder.position(), 9);
    assert_eq!(encoder.get_ref()[1..].to_owned(), e);
}

#[test]
fn test_encode_f32() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    let v = -214648.34432 as f32;
    let e = [0xC8, 0x51, 0x9E, 0x16];
    encoder = encoder.encode_f32(v);

    assert_eq!(encoder.position(), 5);
    assert_eq!(encoder.get_ref()[1..].to_owned(), e);
}

#[test]
fn test_encode_f64() {
    let mut encoder = Cursor::new(Vec::with_capacity(512));
    let v = -922337203685.2345;
    let e = [0xC2, 0x6A, 0xD7, 0xF2, 0x9A, 0xBC, 0xA7, 0x81];
    encoder = encoder.encode_f64(v);

    assert_eq!(encoder.position(), 9);
    assert_eq!(encoder.get_ref()[1..].to_owned(), e);
}