#[derive(Debug)]
pub struct Codec<'a> {
    pub name: &'a str,
    pub lossless: bool,
    pub extension: &'a str,
}
