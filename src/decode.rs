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
fn split_chunk(chunk: [u8; 4]) -> [u8; 3] {
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
            let mut decoded_chunk = [0u8; 4];

            let nvalid = 0;
            for (nvalid, &byte) in chunk.iter().enumerate() {
                if byte == b'=' {
                    break;
                }
                decoded_chunk[nvalid] = decode_byte(byte)?;
            }

            let decoded = split_chunk(decoded_chunk);
            let output_len = (nvalid * 3) / 4; // 4→3, 3→2, 2→1
            writer.write_all(&split_chunk(decoded_chunk)[..output_len])?;
            writer.write_all(&decoded)?;
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
