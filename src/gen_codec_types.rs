extern crate phf_codegen;
extern crate unicase;

use phf_codegen::Map as PhfMap;
use unicase::UniCase;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

use codecs::ALL_CODECS;

const GENERATED_FILE: &'static str = "src/codecs_generated.rs";

mod codecs;


fn main() {
    let mut outfile = BufWriter::new(File::create(GENERATED_FILE).unwrap());

    build_map(&mut outfile);
}

fn build_map<W: Write>(out: &mut W) {
    write!(out, "static ALL_CODECS: phf::Map<UniCase<&'static str>, (bool, &'static str)> = ").unwrap();
    let mut forward_map = PhfMap::new();

    for &(name, lossless, extension) in ALL_CODECS {
        forward_map.entry(UniCase(name), &format!("{:?}", (lossless, extension)));
    }

    forward_map.build(out).unwrap();

    writeln!(out, ";").unwrap();
}
