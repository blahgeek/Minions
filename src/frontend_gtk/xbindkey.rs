/*
* @Author: BlahGeek
* @Date:   2017-07-01
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-07-01
*/

extern crate nix;

use std;
use std::io::Write;
use std::fs::File;
use std::thread;
use std::process::{Command, Stdio, Child};


pub fn bindkeys<F>(callback: F) -> Child
    where F: Send + 'static + FnMut(bool) {

    let mut config = std::env::temp_dir();
    config.push("minions-xbindkeysrc");

    {
        let mut config = File::create(&config).expect("Unable to create tmp file");
        let s = include_str!("./resource/xbindkeysrc");
        let s = s.replace("{}", &format!("{}", nix::unistd::getpid()));
        config.write_all(s.as_bytes()).expect("Unable to write to tmp file");
    }

    thread::spawn(move || {
        let mut callback = callback;

        let mut sigmask = nix::sys::signal::SigSet::empty();
        sigmask.add(nix::sys::signal::Signal::SIGUSR1);
        sigmask.add(nix::sys::signal::Signal::SIGUSR2);

        loop {
            match sigmask.wait() {
                Ok(nix::sys::signal::Signal::SIGUSR1) => {
                    debug!("Got signal: USR1");
                    callback(false);
                },
                Ok(nix::sys::signal::Signal::SIGUSR2) => {
                    debug!("Got signal: USR2");
                    callback(true);
                },
                Ok(sig) => {
                    debug!("Unknown signal: {:?}", sig);
                },
                Err(error) => {
                    warn!("Signal wait error: {}", error);
                }
            }
        }
    });

    Command::new("xbindkeys")
            .arg("-n")
            .arg("-f").arg(&config)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()

}
