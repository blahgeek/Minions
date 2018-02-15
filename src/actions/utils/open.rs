use std::error::Error;

use actions::utils::subprocess;

pub fn that(path: &str) -> Result<(), Box<Error + Sync + Send>> {
    info!("Opening URL: {}", path);
    let args: Vec<&str> = vec![path];
    subprocess::spawn("xdg-open", &args)
}
