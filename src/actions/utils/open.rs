use std::io::Result;
use actions::utils::subprocess;

pub fn that(path: &str) -> Result<()> {
    info!("Opening URL: {}", path);
    let args: Vec<&str> = vec![path];
    subprocess::spawn("xdg-open", &args)
}
