use std::io::{self, BufReader, Read};

const BASE64_CHARS: &[u8] = concat!(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ", // uppercase
    "abcdefghijklmnopqrstuvwxyz", // lowercase
    "0123456789",                 // digits
    "+/"                          // special
)
.as_bytes();
const BUFSIZE: usize = 4096;

#[inline]
fn byte_to_char(b: usize) -> char {
    BASE64_CHARS[b] as char
}

#[inline]
fn split_chunk(chunk: &[u8; 3]) -> [usize; 4] {
    let [a, b, c] = chunk.map(|n| n as usize);
    [
        a >> 2,
        ((a & 0x03) << 4) | (b >> 4),
        ((b & 0x0F) << 2) | (c >> 6),
        c & 0x3F,
    ]
}

#[inline]
fn encode_remainder(remainder: Vec<u8>) -> String {
    assert!(
        remainder.len() < 3,
        "Remainder cannot be longer than 2 bytes"
    );

    let mut padded = [0u8; 3];
    padded[..remainder.len()].copy_from_slice(&remainder);

    let [a, b, c, _] = split_chunk(&padded).map(|n| byte_to_char(n));

    if remainder.len() == 2 {
        format!("{}{}{}=", a, b, c)
    } else {
        format!("{}{}==", a, b)
    }
}

fn main() -> std::io::Result<()> {
    let mut reader = BufReader::new(io::stdin().lock());
    let mut buffer = [0u8; BUFSIZE];
    let mut remainder = Vec::new();

    loop {
        let nread = reader.read(&mut buffer)?;
        if nread == 0 {
            break; // EOF
        };

        let chunks = buffer[..nread].chunks_exact(3);
        for chunk in chunks.clone() {
            let c = chunk.try_into().unwrap();
            let [a, b, c, d] = split_chunk(c).map(|n| byte_to_char(n));
            print!("{}{}{}{}", a, b, c, d);
        }

        remainder = chunks.remainder().to_vec();
    }

    if !remainder.is_empty() {
        print!("{}", encode_remainder(remainder));
    }

    // trailing newline
    print!("\n");

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_byte_to_char() {
        assert_eq!('A', byte_to_char(0));
        assert_eq!('Z', byte_to_char(25));
        assert_eq!('a', byte_to_char(26));
        assert_eq!('z', byte_to_char(51));
        assert_eq!('0', byte_to_char(52));
        assert_eq!('9', byte_to_char(61));
        assert_eq!('+', byte_to_char(62));
        assert_eq!('/', byte_to_char(63));
    }

    #[test]
    fn test_split_chunk() {
        let chunk = [b'M', b'a', b'n'];
        assert_eq!([19, 22, 5, 46], split_chunk(&chunk));

        let chunk = [0, 0, 0];
        assert_eq!([0, 0, 0, 0], split_chunk(&chunk));

        let chunk = [255, 255, 255];
        assert_eq!([63, 63, 63, 63], split_chunk(&chunk));
    }

    #[test]
    fn test_encode_remainder() {
        let remainder = Vec::from([0]);
        assert_eq!("AA==", encode_remainder(remainder));

        let remainder = Vec::from([0, 0]);
        assert_eq!("AAA=", encode_remainder(remainder));
    }

    #[test]
    #[should_panic]
    fn test_encode_remainder_invalid() {
        let remainder = Vec::from([0, 0, 0]);
        encode_remainder(remainder);
    }
}
