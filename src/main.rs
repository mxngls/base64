use rs_base64::decode::decode_stream;
use rs_base64::encode::encode_stream;

use std::{
    env,
    error::Error,
    io::{self, BufReader},
};

fn print_help(program_name: String) {
    print!(
        "{} - Base64 encoder/decoder

USAGE:
    rs-base64 [OPTIONS]

DESCRIPTION
    Encodes, or optionally decodes, data from stdin to stdout.

OPTIONS:
    -d, --decode    Decode instead of encode
    -h, --help      Print this help message
",
        program_name
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    let program_name = args.next().unwrap_or_else(|| "rs-base64".to_string());

    let (mut show_help, mut decode) = (false, false);
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => show_help = true,
            "-d" | "--decode" => decode = true,
            _ => {}
        }
    }

    if show_help {
        print_help(program_name);
        return Ok(());
    }

    let reader = BufReader::new(io::stdin().lock());

    if decode {
        decode_stream(reader, io::stdout())?;
    } else {
        encode_stream(reader, io::stdout())?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_decode_equals_encode() {
        let original = b"Hello, World!";

        let mut encoded = Vec::new();
        let mut decoded = Vec::new();

        encode_stream(Cursor::new(original), &mut encoded).unwrap();
        decode_stream(Cursor::new(&encoded), &mut decoded).unwrap();

        assert_eq!(original.to_vec(), decoded);
    }
}
