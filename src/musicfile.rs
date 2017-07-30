use std::path::PathBuf;

pub struct Musicfile {
    filename: PathBuf,
}

impl Musicfile {
    pub fn new(filename: PathBuf) -> Musicfile {
        Musicfile {
            filename
        }
    }
}
