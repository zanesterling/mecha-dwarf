// This is a module to decode ULEB128 and SLEB128 numbers.
//
// The Wiki article gives a good description of the format:
//   https://en.wikipedia.org/wiki/LEB128
// You can also find it documented in the DWARF documents at dwarfstd.org.

pub fn uleb128_encode(mut n: u64) -> Box<[u8]> {
    if n == 0 { return Box::new([0]) }
    let mut out = vec![];
    while n > 0 {
        let data128 = (n & 0x7f) as u8; // get 7 bits
        out.push(data128 | 0x80); // set top bit: there's another byte
        n >>= 7;
    }
    let len = out.len();
    let last_byte = out[len - 1];
    out[len - 1] = last_byte & 0x7f; // zero last byte's top bit
    out.into_boxed_slice()
}

pub fn uleb128_decode(bytes: Box<[u8]>) -> u64 {
    let mut val: u64 = 0;
    for b in (*bytes).into_iter().rev() {
        val = (val << 7) | (b & 0x7f) as u64;
    }
    return val;
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
        assert_eq!(uleb128_decode(Box::new([2])),            2);
        assert_eq!(uleb128_decode(Box::new([127])),          127);
        assert_eq!(uleb128_decode(Box::new([0x80|0,  1])),   128);
        assert_eq!(uleb128_decode(Box::new([0x80|1,  1])),   129);
        assert_eq!(uleb128_decode(Box::new([0x80|2,  1])),   130);
        assert_eq!(uleb128_decode(Box::new([0x80|57, 100])), 12857);
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
}
