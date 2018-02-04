/*
* @Author: BlahGeek
* @Date:   2017-06-20
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-02-04
*/

extern crate minions;
extern crate toml;
extern crate clap;
extern crate nix;
extern crate gtk;
extern crate chrono;

extern crate log;
extern crate fern;

use std::env;
use std::path::Path;

use minions::frontend::app::MinionsApp;
use minions::mcore::matcher::Matcher;
use minions::mcore::config::Config;

fn main() {
    let mut logger = fern::Dispatch::new()
                         .level(log::LogLevelFilter::Warn);

    let args = clap::App::new("Minions")
                        .author("BlahGeek <i@blahgeek.com>")
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

    let configfile = match args.value_of("config") {
        Some(filename) => Path::new(&filename).to_path_buf(),
        None => env::home_dir().unwrap().join(".minions/config.toml"),
    };
    let config = Config::new(&configfile);

    let history_path = env::home_dir().unwrap().join(".minions/history.dat");
    let matcher = Matcher::new(&history_path, &config.get::<String>(&["core", "history_file_salt"]).unwrap())
                  .expect("Unable to load history file");

    gtk::init().expect("Failed to initialize GTK");
    let _ = MinionsApp::new(&config, matcher);
    gtk::main();
}
