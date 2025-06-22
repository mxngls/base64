use std::{
    error::Error,
    io::{Cursor, Read, Write},
};

#[rustfmt::skip]
const BASE64_INT: [u8; 256] = [
//    1    2    3    4    5    6    7    8    9   10
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 62,  255, 255, 255, 63,  52,  53,
     54,  55,  56,  57,  58,  59,  60,  61, 255, 255,
    255, 255, 255, 255, 255,   0,   1,   2,   3,   4,
      5,   6,   7,   8,   9,  10,  11,  12,  13,  14,
     15,  16,  17,  18,  19,  20,  21,  22,  23,  24,
     25, 255, 255, 255, 255, 255, 255,  26,  27,  28,
     29,  30,  31,  32,  33,  34,  35,  36,  37,  38,
     39,  40,  41,  42,  43,  44,  45,  46,  47,  48,
     49,  50,  51, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255
];

const BUFSIZE: usize = 4096;

#[inline]
fn decode_byte(c: u8) -> Result<u8, &'static str> {
    let value = BASE64_INT[c as usize];
    if value == 255 {
        Err("Invalid character encountered")
    } else {
        Ok(value)
    }
}

#[inline]
fn coerce_chunk(chunk: [u8; 4]) -> [u8; 3] {
    [
        chunk[0] << 2 | chunk[1] >> 4,
        chunk[1] << 4 | chunk[2] >> 2,
        chunk[2] << 6 | chunk[3],
    ]
}

pub fn decode_stream<R: Read, W: Write>(
    mut reader: R,
    mut writer: W,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0u8; BUFSIZE];
    let mut remainder = Vec::new();

    loop {
        let nread = reader.read(&mut buffer)?;
        if nread == 0 {
            break; // EOF
        }

        let mut data = remainder;
        data.extend_from_slice(&buffer[..nread]);

        let chunks = data.chunks_exact(4);
        remainder = chunks.remainder().to_vec();

        for chunk in chunks {
            let chunk: [u8; 4] = chunk.try_into().unwrap();
            let nvalid = chunk.iter().position(|b| *b == b'=').unwrap_or(4);

            let mut decoded = [0u8; 4];
            for i in 0..nvalid {
                decoded[i] = decode_byte(chunk[i])?;
            }

            let chunk = coerce_chunk(decoded);
            writer.write_all(&chunk[..(nvalid * 3) / 4])?;
        }
    }

    assert!(remainder.is_empty(), "Received input of invalid length");

    Ok(())
}

pub fn decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, Box<dyn Error>> {
    let bytes = input.as_ref();
    let mut output = Vec::new();

    decode_stream(Cursor::new(bytes), &mut output)?;

    Ok(output)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decode_byte() {
        assert_eq!(0, decode_byte(b'A').unwrap());
        assert_eq!(25, decode_byte(b'Z').unwrap());
        assert_eq!(26, decode_byte(b'a').unwrap());
        assert_eq!(51, decode_byte(b'z').unwrap());
        assert_eq!(52, decode_byte(b'0').unwrap());
        assert_eq!(61, decode_byte(b'9').unwrap());
        assert_eq!(62, decode_byte(b'+').unwrap());
        assert_eq!(63, decode_byte(b'/').unwrap());
    }

    #[test]
    fn test_coerce_chunk() {
        assert_eq!([b'M', b'a', b'n'], coerce_chunk([19, 22, 5, 46]));
        assert_eq!([0, 0, 0], coerce_chunk([0, 0, 0, 0]));
        assert_eq!([255, 255, 255], coerce_chunk([63, 63, 63, 63]))
    }

    #[test]
    fn test_decode() {
        // see https://datatracker.ietf.org/doc/html/rfc4648#section-10
        assert_eq!(b"".to_vec(), decode("").unwrap());
        assert_eq!(b"f".to_vec(), decode("Zg==").unwrap());
        assert_eq!(b"fo".to_vec(), decode("Zm8=").unwrap());
        assert_eq!(b"foo".to_vec(), decode("Zm9v").unwrap());
        assert_eq!(b"foob".to_vec(), decode("Zm9vYg==").unwrap());
        assert_eq!(b"fooba".to_vec(), decode("Zm9vYmE=").unwrap());
        assert_eq!(b"foobar".to_vec(), decode("Zm9vYmFy").unwrap());
    }
}
