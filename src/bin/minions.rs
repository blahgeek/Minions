/*
* @Author: BlahGeek
* @Date:   2017-06-20
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-07-01
*/

extern crate minions;
extern crate env_logger;
extern crate toml;
extern crate clap;

#[macro_use]
extern crate log;

#[cfg(feature="use-gtk")]
extern crate gtk;

use std::fs::File;
use std::io::prelude::*;

fn run_rofi_app(config: toml::Value, from_clipboard: bool) {
    let mut app = minions::frontend_rofi::app::MinionsApp::new(config, from_clipboard);
    app.run_loop();
}

#[cfg(feature="use-gtk")]
fn run_gtk_app(config: toml::Value) {
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
    let from_clipboard = args.is_present("from_clipboard");

    if args.is_present("rofi") {
        run_rofi_app(config, from_clipboard)
    } else {
        run_gtk_app(config)
    }
}
