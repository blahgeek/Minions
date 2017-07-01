/*
* @Author: BlahGeek
* @Date:   2017-07-01
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-07-01
*/

use std;
use std::io::{Read, Write};
use std::fs::File;
use std::thread;
use std::process::{Command, Stdio};


fn bindkeys<F>(callback: F)
    where F: Send + 'static + FnMut(bool) -> bool {

    let mut config = std::env::temp_dir();
    config.push("minions-xbindkeysrc");

    {
        let mut config = File::create(&config).expect("Unable to create tmp file");
        config.write_all(include_bytes!("./resource/xbindkeysrc")).expect("Unable to write to tmp file");
    }

    thread::spawn(move || {
        let mut callback = callback;
        let child = Command::new("xbindkeys")
                                .arg("-n")
                                .arg("-f").arg(&config)
                                .stdout(Stdio::piped())
                                .spawn()
                                .expect("Unable to spawn xbindkeys");
        let stdout = child.stdout.unwrap().bytes();
        for ch in stdout {
            let ch = ch.unwrap();
            debug!("Output from xbindkeys: {}", ch);
            let should_continue =
                if ch == ('S' as u8) { callback(false) }
                else if ch == ('P' as u8) { callback(true) }
                else { true };
            if !should_continue {
                break;
            }
        }
    });
}
