extern crate casm;

use casm::errors::Error;
use std::process;

fn main() {
    if let Err(ref err) = casm::run() {
        handle_error(err);
    }
}

fn handle_error(e: & Error ) {
    use std::io::Write;
    let stderr = &mut ::std::io::stderr();
    let errmsg = "Error writing to stderr";
    writeln!(stderr, "error: {}", e).expect(errmsg);

    for e in e.iter().skip(1) {
        writeln!(stderr, "caused by: {}", e).expect(errmsg);
    }

    // The backtrace is not always generated. Try to run this example
    // with `RUST_BACKTRACE=1`.
    if let Some(backtrace) = e.backtrace() {
        writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
    }

    ::process::exit(1);
}
