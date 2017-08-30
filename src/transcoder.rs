extern crate ffmpeg;

use std::iter::FromIterator;
use std::path::Path;

use ffmpeg::{format, codec, frame, media, filter};

fn filter(
    spec: &str,
    decoder: &codec::decoder::Audio,
    encoder: &codec::encoder::Audio,
) -> Result<filter::Graph, ffmpeg::Error> {
    let mut filter = filter::Graph::new();

    let args = format!(
        "time_base={}:sample_rate={}:sample_fmt={}:channel_layout=0x{:x}",
        decoder.time_base(),
        decoder.rate(),
        decoder.format().name(),
        decoder.channel_layout().bits()
    );

    try!(filter.add(&filter::find("abuffer").unwrap(), "in", &args));
    try!(filter.add(&filter::find("abuffersink").unwrap(), "out", ""));

    {
        let mut out = filter.get("out").unwrap();

        out.set_sample_format(encoder.format());
        out.set_channel_layout(encoder.channel_layout());
        out.set_sample_rate(encoder.rate());
    }

    try!(try!(try!(filter.output("in", 0)).input("out", 0)).parse(
        spec,
    ));
    try!(filter.validate());

    if let Some(codec) = encoder.codec() {
        if !codec.capabilities().contains(
            ffmpeg::codec::capabilities::VARIABLE_FRAME_SIZE,
        )
        {
            filter.get("out").unwrap().sink().set_frame_size(
                encoder.frame_size(),
            );
        }
    }

    Ok(filter)
}

struct Transcoder {
    stream: usize,
    filter: filter::Graph,
    decoder: codec::decoder::Audio,
    encoder: codec::encoder::Audio,
}

fn transcoder<P: AsRef<Path>>(
    ictx: &mut format::context::Input,
    octx: &mut format::context::Output,
    path: &P,
    filter_spec: &str,
    bit_rate: usize,
) -> Result<Transcoder, ffmpeg::Error> {
    let input = ictx.streams().best(media::Type::Audio).expect(
        "could not find best audio stream",
    );
    let mut decoder = try!(input.codec().decoder().audio());
    let codec = try!(
        ffmpeg::encoder::find(octx.format().codec(path, media::Type::Audio))
            .expect("failed to find encoder")
            .audio()
    );
    let global = octx.format().flags().contains(
        ffmpeg::format::flag::GLOBAL_HEADER,
    );

    try!(decoder.set_parameters(input.parameters()));

    let mut output = try!(octx.add_stream(codec));
    let mut encoder = try!(output.codec().encoder().audio());

    let channel_layout = codec
        .channel_layouts()
        .map(|cls| cls.best(decoder.channel_layout().channels()))
        .unwrap_or(ffmpeg::channel_layout::STEREO);

    if global {
        encoder.set_flags(ffmpeg::codec::flag::GLOBAL_HEADER);
    }

    encoder.set_rate(select_best_rate(decoder.rate() as i32, codec.rates()));
    encoder.set_channel_layout(channel_layout);
    encoder.set_channels(channel_layout.channels());
    encoder.set_format(
        codec
            .formats()
            .expect("unknown supported formats")
            .next()
            .unwrap(),
    );
    encoder.set_bit_rate(bit_rate);

    encoder.set_time_base((1, decoder.rate() as i32));
    output.set_time_base((1, decoder.rate() as i32));

    let encoder = try!(encoder.open_as(codec));
    output.set_parameters(&encoder);

    let filter = try!(filter(filter_spec, &decoder, &encoder));

    Ok(Transcoder {
        stream: input.index(),
        filter: filter,
        decoder: decoder,
        encoder: encoder,
    })
}

pub fn convert(input: &str, output: &str, filter: &str, bit_rate: usize) {
    ffmpeg::init().unwrap();

    let mut ictx = format::input(&input).unwrap();
    let mut octx = format::output(&output).unwrap();
    let mut transcoder = transcoder(&mut ictx, &mut octx, &output, filter, bit_rate).unwrap();

    octx.set_metadata(ictx.metadata().to_owned());
    octx.write_header().unwrap();

    let in_time_base = transcoder.decoder.time_base();
    let out_time_base = octx.stream(0).unwrap().time_base();

    let mut decoded = frame::Audio::empty();
    let mut encoded = ffmpeg::Packet::empty();

    for (stream, mut packet) in ictx.packets() {
        if stream.index() == transcoder.stream {
            packet.rescale_ts(stream.time_base(), in_time_base);

            if let Ok(true) = transcoder.decoder.decode(&packet, &mut decoded) {
                let timestamp = decoded.timestamp();
                decoded.set_pts(timestamp);

                transcoder
                    .filter
                    .get("in")
                    .unwrap()
                    .source()
                    .add(&decoded)
                    .unwrap();

                while let Ok(..) = transcoder.filter.get("out").unwrap().sink().frame(
                    &mut decoded,
                )
                {
                    if let Ok(true) = transcoder.encoder.encode(&decoded, &mut encoded) {
                        encoded.set_stream(0);
                        encoded.rescale_ts(in_time_base, out_time_base);
                        encoded.write_interleaved(&mut octx).unwrap();
                    }
                }
            }
        }
    }

    transcoder
        .filter
        .get("in")
        .unwrap()
        .source()
        .flush()
        .unwrap();

    while let Ok(..) = transcoder.filter.get("out").unwrap().sink().frame(
        &mut decoded,
    )
    {
        if let Ok(true) = transcoder.encoder.encode(&decoded, &mut encoded) {
            encoded.set_stream(0);
            encoded.rescale_ts(in_time_base, out_time_base);
            encoded.write_interleaved(&mut octx).unwrap();
        }
    }

    if let Ok(true) = transcoder.encoder.flush(&mut encoded) {
        encoded.set_stream(0);
        encoded.rescale_ts(in_time_base, out_time_base);
        encoded.write_interleaved(&mut octx).unwrap();
    }

    octx.write_trailer().unwrap();
}

/// Get the closest sample rate without going under, unless there is nothing above
/// There are some issues with this.  Sample rate conversion can have some loss, and
/// we're letting the user target a bitrate.  That said, we're mostly doing this to
/// convert from 44.1khz to 48khz for opus.
///
/// This isn't as fast as it could be, but that's ok.  The vectors will be small (<=10).
///
/// # Arguments
///
/// * `source` - The sample rate of the source file
/// * `supported` - The supported sample rates of the target codec
fn select_best_rate(source: i32, supported: Option<codec::audio::RateIter>) -> i32 {
    let mut best_rate = source;
    if let Some(rates) = supported {
        // rates is not always sorted. ie. [44100, 48000, 88200, 96000, 176400, 192000, 0]
        let mut rates = Vec::from_iter(rates);
        rates.sort();
        for rate in rates {
            // If codec doesn't support our rate, get the next highest
            if rate > 0 && rate < source {
                best_rate = rate;
            } else if rate >= source {
                return rate;
            }
        }

    }
    best_rate
}


#[cfg(test)]
mod tests {
    use super::select_best_rate;
    use ffmpeg::codec::audio::RateIter;

    #[test]
    fn test_best_rate_above() {
        let supported_array: [i32; 6] = [48000, 24000, 16000, 12000, 8000, 0];
        let supported = RateIter::new(&supported_array as *const i32);
        let source: i32 = 41000;
        assert_eq!(select_best_rate(source, Some(supported)), 48000);
    }

    #[test]
    fn test_best_rate_below() {
        let supported_array: [i32; 5] = [24000, 16000, 12000, 8000, 0];
        let supported = RateIter::new(&supported_array as *const i32);
        let source: i32 = 41000;
        assert_eq!(select_best_rate(source, Some(supported)), 24000);
    }

    #[test]
    fn test_best_rate_equal() {
        let supported_array: [i32; 7] = [48000, 41000, 24000, 16000, 12000, 8000, 0];
        let supported = RateIter::new(&supported_array as *const i32);
        let source: i32 = 41000;
        assert_eq!(select_best_rate(source, Some(supported)), 41000);
    }
}
