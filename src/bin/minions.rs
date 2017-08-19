/*
* @Author: BlahGeek
* @Date:   2017-06-20
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-08-19
*/

extern crate minions;
extern crate toml;
extern crate clap;
extern crate nix;
extern crate gtk;
extern crate chrono;

#[macro_use]
extern crate log;
extern crate fern;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use minions::frontend::config::GlobalConfig;
use minions::frontend::app::MinionsApp;
use minions::mcore::matcher::Matcher;

fn main() {
    let mut logger = fern::Dispatch::new()
                         .level(log::LogLevelFilter::Warn);

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
                        .arg(clap::Arg::with_name("verbose")
                                      .short("v")
                                      .long("verbose")
                                      .multiple(true)
                                      .help("Increase logging verbosity, up to 2 times"))
                        .get_matches();

    logger = match args.occurrences_of("verbose") {
        0 => logger.level_for("minions", log::LogLevelFilter::Info),
        1 => logger.level_for("minions", log::LogLevelFilter::Debug),
        _ => logger.level_for("minions", log::LogLevelFilter::Trace),
    };
    logger.format(|out, message, record| {
               out.finish(format_args!("{}[{}][{}] {}",
                                       chrono::Local::now().format("[%H:%M:%S]"),
                                       record.level(),
                                       record.target(),
                                       message))
           })
          .chain(std::io::stderr())
          .apply()
          .expect("Unable to setup logging");

    let default_configcontent : String = include_str!("../../config/default.toml").into();
    let default_config = default_configcontent.parse::<toml::Value>().unwrap();

    let configfile = match args.value_of("config") {
        Some(filename) => Path::new(&filename).to_path_buf(),
        None => env::home_dir().unwrap().join(".minions/config.toml"),
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
        info!("Using default builtin config");
        default_config
    } else {
        let mut config = configcontent.parse::<toml::Value>().expect("Invalid config file");
        {
            let config_map = config.as_table_mut().unwrap();
            let default_config = default_config.as_table().unwrap();
            for (key, value) in default_config.into_iter() {
                if !config_map.contains_key(key) {
                    info!("Section {} missing from config, use default", key);
                    config_map.insert(key.clone(), value.clone());
                }
            }
        }
        config
    };

    let global_config = config.get("global").unwrap().clone()
                        .try_into::<GlobalConfig>().expect("Unable to parse global config section");

    let history_path = env::home_dir().unwrap().join(".minions/history.dat");
    let matcher = Matcher::new(&history_path, &global_config.history_file_salt)
                  .expect("Unable to load history file");

    gtk::init().expect("Failed to initialize GTK");
    let _ = MinionsApp::new(global_config, config, matcher);
    gtk::main();
}
