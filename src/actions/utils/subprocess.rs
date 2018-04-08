/*
* @Author: BlahGeek
* @Date:   2017-07-07
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-04-08
*/

extern crate nix;

use std::ffi::CString;
use std::io::Result;


pub fn spawn(cmd: &str, args: &[&str]) -> Result<()> {

    let execv_filename = CString::new(cmd)?;
    let mut execv_args = vec![execv_filename.clone()];
    for arg in args {
        let arg = arg.to_string();
        execv_args.push(CString::new(arg.as_str())?);
    }

    // fork once
    match nix::unistd::fork().expect("Fork failed") {
        nix::unistd::ForkResult::Parent{child, ..} => {
            debug!("Waiting for child {}", child);
            nix::sys::wait::waitpid(child, None).unwrap();
        },
        nix::unistd::ForkResult::Child => {
            // daemonize (fork again and setsid)
            nix::unistd::daemon(false, false).expect("Daemonize failed");
            let _ = nix::unistd::execvp(&execv_filename, &execv_args);
        },
    }

    Ok(())
}
