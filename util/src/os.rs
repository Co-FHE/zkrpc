use std::fs::{self, File};
use std::io::Error;
use std::path::Path;

pub fn create_file_and_dirs(path: &str) -> Result<(), Error> {
    let path = Path::new(path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    File::create(path)?;
    Ok(())
}
