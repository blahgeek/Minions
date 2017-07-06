/*
* @Author: BlahGeek
* @Date:   2017-07-07
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-07-07
*/

extern crate nix;

use std;
use std::error::Error;
use std::ffi::CString;


pub fn spawn(cmd: &str, args: &[&str]) -> Result<(), Box<Error + Sync + Send>> {

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
            // fork again
            match nix::unistd::fork().expect("Fork failed") {
                nix::unistd::ForkResult::Child => {
                    let _ = nix::unistd::execvp(&execv_filename, &execv_args);
                },
                nix::unistd::ForkResult::Parent {..} => {
                    std::process::exit(0);
                }
            };
        },
    }

    Ok(())
}
