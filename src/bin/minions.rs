/*
* @Author: BlahGeek
* @Date:   2017-06-20
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-07-08
*/

extern crate minions;
extern crate env_logger;
extern crate toml;
extern crate clap;
extern crate nix;

#[macro_use]
extern crate log;

#[cfg(feature="use-gtk")]
extern crate gtk;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

fn run_rofi_app(config: toml::Value, from_clipboard: bool) {
    let mut app = minions::frontend_rofi::app::MinionsApp::new(config, from_clipboard);
    app.run_loop();
}

#[cfg(feature="use-gtk")]
fn run_gtk_app(config: toml::Value) {

    // block USR1 and USR2, used for window trigger
    let mut sigmask = nix::sys::signal::SigSet::empty();
    sigmask.add(nix::sys::signal::Signal::SIGUSR1);
    sigmask.add(nix::sys::signal::Signal::SIGUSR2);
    sigmask.thread_block().expect("Error block signals");

    gtk::init().expect("Failed to initialize GTK");
    let _ = minions::frontend_gtk::app::MinionsApp::new(config);
    gtk::main();
}

#[cfg(not(feature="use-gtk"))]
fn run_gtk_app(_: toml::Value, _: bool) {
    panic!("GTK frontend unavailable");
}

fn main() {
    env_logger::init().unwrap();

    let args = clap::App::new("Minions (rofi frontend)")
                        .author("BlahGeek <i@blahgeek.com>")
                        .arg(clap::Arg::with_name("rofi")
                                      .long("rofi")
                                      .help("Use rofi frontend"))
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

    let default_configcontent : String = include_str!("../../config/default.toml").into();
    let default_config = default_configcontent.parse::<toml::Value>().unwrap();

    let configfile = match args.value_of("config") {
        Some(filename) => Path::new(&filename).to_path_buf(),
        None => env::home_dir().unwrap().join(".minions.toml"),
    };
    let mut configcontent = String::new();

    {
        info!("Reading config from {:?}", configfile);
        let fin = File::open(&configfile);
        if let Ok(mut fin) = fin {
            let _ = fin.read_to_string(&mut configcontent);
        }
    }

    let config = if configcontent.len() == 0 {
        warn!("Using default builtin config");
        default_config
    } else {
        let mut config = configcontent.parse::<toml::Value>().expect("Invalid config file");
        {
            let config_map = config.as_table_mut().unwrap();
            let default_config = default_config.as_table().unwrap();
            for (key, value) in default_config.into_iter() {
                if !config_map.contains_key(key) {
                    warn!("Section {} missing from config, use default", key);
                    config_map.insert(key.clone(), value.clone());
                }
            }
        }
        config
    };

    let from_clipboard = args.is_present("from_clipboard");

    if args.is_present("rofi") {
        run_rofi_app(config, from_clipboard)
    } else {
        run_gtk_app(config)
    }
}
