// COPIED FROM https://github.com/Byron/open-rs/blob/master/src/lib.rs
// WITH SOME MIDIFICATIONS:
//  - do not wait
use std::io;
use std::process::Command;
use std::ffi::OsStr;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn that<T:AsRef<OsStr>+Sized>(path: T) -> io::Result<()> {
    let mut last_err: io::Result<()> = Err(io::Error::from_raw_os_error(0));
    for program in &["xdg-open", "gnome-open", "kde-open"] {
        match Command::new(program).arg(path.as_ref()).spawn() {
            Ok(_) => return Ok(()),
            Err(err) => {
                last_err = Err(err);
                continue;
            },
        }
    }
    last_err
}

#[cfg(target_os = "windows")]
pub fn that<T:AsRef<OsStr>+Sized>(path: T) -> io::Result<()> {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C").arg("start");
    if let Some(s) = path.as_ref().to_str() {
        cmd.arg(s.replace("&", "^&"));
    } else {
        cmd.arg(path.as_ref());
    }
    cmd.spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn that<T:AsRef<OsStr>+Sized>(path: T) -> io::Result<()> {
    Command::new("open").arg(path.as_ref()).spawn()?;
    Ok(())
}
