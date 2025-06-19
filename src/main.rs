use std::{
    error::Error,
    io::{self, BufReader},
};

use rs_base64::encode_stream;

fn main() -> Result<(), Box<dyn Error>> {
    let reader = BufReader::new(io::stdin().lock());

    encode_stream(reader, io::stdout())?;

    // trailing newline;
    println!();

    Ok(())
}
