use std::io;
use std::error::Error;
use std::process::Command;
use std::ffi::OsStr;

use actions::utils::subprocess;

pub fn that(path: &str) -> Result<(), Box<Error + Sync + Send>> {
    let args: Vec<&str> = vec![path];
    subprocess::spawn("xdg-open", &args)
}
