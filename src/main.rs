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
    let [a, b, c] = chunk;
    [
        (a >> 2) as usize,
        (((a & 0x03) << 4) | (b >> 4)) as usize,
        (((b & 0x0F) << 2) | (c >> 6)) as usize,
        (c & 0x3F) as usize,
    ]
}

fn main() -> std::io::Result<()> {
    let mut reader = BufReader::new(io::stdin().lock());
    let mut buffer = [0u8; BUFSIZE];
    let mut left = Vec::new();

    loop {
        let nread = reader.read(&mut buffer)?;
        if nread == 0 {
            break; // EOF
        };

        let chunks = buffer[..nread].chunks_exact(3);
        for chunk in chunks.clone() {
            let [a, b, c, d] = split_chunk(chunk.try_into().unwrap());
            print!(
                "{}{}{}{}",
                byte_to_char(a),
                byte_to_char(b),
                byte_to_char(c),
                byte_to_char(d)
            );
        }

        left = chunks.remainder().to_vec();
    }

    if !left.is_empty() {
        let mut padded = [0u8; 3];
        padded[..left.len()].copy_from_slice(&left);
        let [a, b, c, _] = split_chunk(&padded);

        if left.len() == 2 {
            print!("{}{}{}=", byte_to_char(a), byte_to_char(b), byte_to_char(c));
        } else {
            print!("{}{}==", byte_to_char(a), byte_to_char(b));
        }
    }

    // trailing newline
    print!("\n");

    Ok(())
}
