/*
* @Author: BlahGeek
* @Date:   2017-06-13
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-18
*/

extern crate minions;
extern crate env_logger;
extern crate toml;
extern crate clap;

#[macro_use]
extern crate log;

use std::fs::File;
use std::io::prelude::*;

fn main() {
    env_logger::init().unwrap();

    let args = clap::App::new("Minions (rofi frontend)")
                        .author("BlahGeek <i@blahgeek.com>")
                        .arg(clap::Arg::with_name("config")
                                      .short("c")
                                      .long("config")
                                      .help("Config (TOML) file to use")
                                      .takes_value(true))
                        .arg(clap::Arg::with_name("from_clipboard")
                                      .short("f")
                                      .long("from-clipboard")
                                      .help("Quicksend content from clipboard"))
                        .get_matches();

    let configfile = args.value_of("config");
    let mut configcontent = String::new();
    if let Some(configfile) = configfile {
        info!("Reading config from {}", configfile);
        let fin = File::open(&configfile);
        if let Ok(mut fin) = fin {
            let _ = fin.read_to_string(&mut configcontent);
        }
    }
    if configcontent.len() == 0 {
        warn!("Using default builtin config");
        configcontent = include_str!("../../config/example.toml").into();
    }

    let config = configcontent.parse::<toml::Value>().expect("Invalid config file");

    let mut app = minions::frontend_rofi::app::MinionsApp::new(config, args.is_present("from_clipboard"));
    app.run_loop();
}
