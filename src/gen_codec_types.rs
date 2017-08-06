extern crate phf_codegen;
extern crate unicase;

use phf_codegen::Map as PhfMap;
use unicase::UniCase;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

const GENERATED_FILE: &'static str = "src/codecs_generated.rs";

mod codec;


fn main() {
    let mut outfile = BufWriter::new(File::create(GENERATED_FILE).unwrap());

    build_map(&mut outfile);
}

fn build_map<W: Write>(out: &mut W) {
    write!(out,
           "static ALL_CODECS: phf::Map<UniCase<&'static str>, codec::Codec> = ")
        .unwrap();
    let mut forward_map = PhfMap::new();

    for &(name, lossless, extension) in ALL_CODECS {
        forward_map.entry(UniCase(name),
                          &format!("codec::Codec{{ name: \"{}\", lossless: {}, extension: \
                                    \"{}\"}}",
                                   name,
                                   lossless,
                                   extension));
    }

    forward_map.build(out).unwrap();

    writeln!(out, ";").unwrap();
}

// I have removed the ffmpeg codecs that will never ever be used (i.e. they existed in one game)
#[cfg_attr(rustfmt, rustfmt_skip)]
pub static ALL_CODECS: &'static [(&'static str, bool, &'static str)] = &[
    ("AAC",              false, "m4a"  ),
    ("AAC_LATM",         false, "m4a"  ),
    ("AC3",              false, "ac3"  ),
    ("ALAC",             true,  "m4a"  ),
    ("APE",              false, "ape"  ),
    ("ATRAC1",           false, "aea"  ),
    ("ATRAC3",           false, "at3"  ),
    ("ATRAC3P",          false, "aa3"  ),
    ("BINKAUDIO_DCT",    false, "bik"  ),
    ("BINKAUDIO_RDFT",   false, "bik"  ),
    ("CELT",             false, "ogg"  ),
    ("COOK",             false, "rm"   ),
    ("DSD_LSBF",         true,  "dff"  ),
    ("DSD_LSBF_PLANAR",  true,  "dff"  ),
    ("DSD_MSBF",         true,  "dff"  ),
    ("DSD_MSBF_PLANAR",  true,  "dff"  ),
    ("DSS_SP",           false, "dss"  ),
    ("DTS",              false, "dts"  ),
    ("EAC3",             false, "eac3" ),
    ("FLAC",             true,  "flac" ),
    ("IAC",              false, "avi"  ),
    ("IMC",              false, "avi"  ),
    ("MLP",              true,  "mlp"  ),
    ("MP1",              false, "mp1"  ),
    ("MP2",              false, "mp2"  ),
    ("MP3",              false, "mp3"  ),
    ("MP3ADU",           false, "mp3"  ),
    ("MP3ON4",           false, "mp3"  ),
    ("MP4ALS",           true,  "m4a"  ),
    ("MUSEPACK7",        false, "mpc"  ),
    ("MUSEPACK8",        false, "mpc"  ),
    ("OPUS",             false, "opus" ),
    ("QCELP",            false, "qcp"  ),
    ("QDM2",             false, "mov"  ),
    ("QDMC",             false, "mov"  ),
    ("RALF",             true,  "rmvb" ),
    ("SHORTEN",          true,  "shn"  ),
    ("SIPR",             false, "rm"   ),
    ("SPEEX",            false, "spx"  ),
    ("TAK",              true,  "tak"  ),
    ("TTA",              true,  "tta"  ),
    ("TWINVQ",           false, "vqf"  ),
    ("VORBIS",           false, "ogg"  ),
    ("WAVPACK",          true,  ".wv"  ),
    ("WMALOSSLESS",      true,  "wma"  ),
    ("WMAPRO",           false, "wma"  ),
    ("WMAV1",            false, "wma"  ),
    ("WMAV2",            false, "wma"  ),
    ("WMAVOICE",         false, "wma"  ),
    ("PCM_S16LE",        true,  "wav"  ),
    ("PCM_S16BE",        true,  "wav"  ),
    ("PCM_U16LE",        true,  "wav"  ),
    ("PCM_U16BE",        true,  "wav"  ),
    ("PCM_S8",           true,  "wav"  ),
    ("PCM_U8",           true,  "wav"  ),
    ("PCM_MULAW",        true,  "wav"  ),
    ("PCM_ALAW",         true,  "wav"  ),
    ("PCM_S32LE",        true,  "wav"  ),
    ("PCM_S32BE",        true,  "wav"  ),
    ("PCM_U32LE",        true,  "wav"  ),
    ("PCM_U32BE",        true,  "wav"  ),
    ("PCM_S24LE",        true,  "wav"  ),
    ("PCM_S24BE",        true,  "wav"  ),
    ("PCM_U24LE",        true,  "wav"  ),
    ("PCM_U24BE",        true,  "wav"  ),
    ("PCM_S24DAUD",      true,  "wav"  ),
    ("PCM_ZORK",         true,  "wav"  ),
    ("PCM_S16LE_PLANAR", true,  "wav"  ),
    ("PCM_DVD",          true,  "wav"  ),
    ("PCM_F32BE",        true,  "wav"  ),
    ("PCM_F32LE",        true,  "wav"  ),
    ("PCM_F64BE",        true,  "wav"  ),
    ("PCM_F64LE",        true,  "wav"  ),
    ("PCM_BLURAY",       true,  "wav"  ),
    ("PCM_LXF",          true,  "wav"  ),
    ("S302M",            true,  "wav"  ),
    ("PCM_S8_PLANAR",    true,  "wav"  ),
    ("PCM_S24LE_PLANAR", true,  "wav"  ),
    ("PCM_S32LE_PLANAR", true,  "wav"  ),
    ("PCM_S16BE_PLANAR", true,  "wav"  ),
];
