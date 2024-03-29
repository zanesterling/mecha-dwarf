// This is a module to decode ULEB128 and SLEB128 numbers.
//
// The Wiki article gives a good description of the format:
//   https://en.wikipedia.org/wiki/LEB128
// You can also find it documented in the DWARF documents at dwarfstd.org.

#![allow(dead_code)]


#[derive(PartialEq, Debug)]
pub enum Error {
    LastByteHasContinueBit,
}

impl From<Error> for String {
    fn from(_: Error) -> String {
        "last byte in LEB has continue bit set".to_string()
    }
}

pub fn uleb128_encode(mut n: u64) -> Box<[u8]> {
    let mut out = vec![];
    loop {
        let mut byte = (n as u8 & 0x7f) | 0x80; // get 7 bits; set top bit
        n >>= 7;
        if n == 0 { byte &= 0x7f; }
        out.push(byte);
        if n == 0 { break }
    }
    out.into_boxed_slice()
}

// Reads a ULEB128-encoded value from the input,
// and returns the value and the number of bytes consumed.
pub fn uleb128_decode(bytes: &[u8]) -> Result<(u64, usize), Error> {
    let mut val: u64 = 0;
    let mut shift = 0;
    for (i, b) in bytes.iter().enumerate() {
        let byte = (b & 0x7f) as u64;
        val |= byte << shift;
        if b & 0x80 == 0 { return Ok((val, i+1)); }
        shift += 7;
    }
    Err(Error::LastByteHasContinueBit)
}

pub fn ileb128_encode(mut n: i64) -> Box<[u8]> {
    let mut out = vec![];
    let mut more = true;
    while more {
        let mut byte: u8 = 0x7f & (n as u8);
        n >>= 7;
        if (n == 0  && (byte & 0x40) == 0) ||
           (n == -1 && (byte & 0x40) != 0) {
            more = false;
        } else {
            byte |= 0x80;
        }
        out.push(byte);
    }
    out.into_boxed_slice()
}

pub fn ileb128_decode(bytes: &[u8]) -> Result<(i64, usize), Error> {
    let mut result = 0;
    let mut shift = 0;
    let mut last_byte = 0;
    let mut last_i = 0;
    for (i, b) in bytes.into_iter().enumerate() {
        (last_i, last_byte) = (i, *b);
        let data = (b & 0x7f) as i64;
        result |= data << shift;
        shift += 7;
        if b & 0x80 == 0 { break; }
    }
    if last_byte & 0x80 != 0 {
        return Err(Error::LastByteHasContinueBit);
    }
    // If last byte's sign bit is set..
    if shift < 64 && 0x40 & last_byte != 0 {
        // ..sign extend the result.
        result |= -(1 << shift);
    }
    Ok((result, last_i+1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uleb128_encode_works() {
        assert_eq!(*uleb128_encode(0),     [0]);
        assert_eq!(*uleb128_encode(2),     [2]);
        assert_eq!(*uleb128_encode(127),   [127]);
        assert_eq!(*uleb128_encode(128),   [0x80|0,  1]);
        assert_eq!(*uleb128_encode(129),   [0x80|1,  1]);
        assert_eq!(*uleb128_encode(130),   [0x80|2,  1]);
        assert_eq!(*uleb128_encode(12857), [0x80|57, 100]);
    }

    #[test]
    fn uleb128_decode_works() {
        assert_eq!(uleb128_decode(&[2]),            Ok((2, 1)));
        assert_eq!(uleb128_decode(&[127]),          Ok((127, 1)));
        assert_eq!(uleb128_decode(&[0x80|0,  1]),   Ok((128, 2)));
        assert_eq!(uleb128_decode(&[0x80|1,  1]),   Ok((129, 2)));
        assert_eq!(uleb128_decode(&[0x80|2,  1]),   Ok((130, 2)));
        assert_eq!(uleb128_decode(&[0x80|57, 100]), Ok((12857, 2)));
    }

    #[test]
    fn ileb128_encode_works() {
        assert_eq!(*ileb128_encode(0),    [0]);
        assert_eq!(*ileb128_encode(2),    [2]);
        assert_eq!(*ileb128_encode(127),  [0x80|127,  0]);
        assert_eq!(*ileb128_encode(128),  [0x80|0,    1]);
        assert_eq!(*ileb128_encode(129),  [0x80|1,    1]);
        assert_eq!(*ileb128_encode(-1),   [0x7f]);
        assert_eq!(*ileb128_encode(-2),   [0x7e]);
        assert_eq!(*ileb128_encode(-127), [0x80|1,    0x7f]);
        assert_eq!(*ileb128_encode(-128), [0x80|0,    0x7f]);
        assert_eq!(*ileb128_encode(-129), [0x80|0x7f, 0x7e]);
    }

    #[test]
    fn ileb128_decode_works() {
        assert_eq!(ileb128_decode(&[2]),               Ok(( 2, 1)));
        assert_eq!(ileb128_decode(&[0x80|127,  0]),    Ok(( 127, 2)));
        assert_eq!(ileb128_decode(&[0x80|0,    1]),    Ok(( 128, 2)));
        assert_eq!(ileb128_decode(&[0x80|1,    1]),    Ok(( 129, 2)));
        assert_eq!(ileb128_decode(&[0x7e        ]),    Ok((-2, 1)));
        assert_eq!(ileb128_decode(&[0x80|1,    0x7f]), Ok((-127, 2)));
        assert_eq!(ileb128_decode(&[0x80|0,    0x7f]), Ok((-128, 2)));
        assert_eq!(ileb128_decode(&[0x80|0x7f, 0x7e]), Ok((-129, 2)));
    }
}
