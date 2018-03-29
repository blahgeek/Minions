/*
* @Author: BlahGeek
* @Date:   2017-06-20
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-03-29
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

fn main() {
    let mut logger = fern::Dispatch::new()
                         .level(log::LevelFilter::Warn);

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
        0 => logger.level_for("minions", log::LevelFilter::Info),
        1 => logger.level_for("minions", log::LevelFilter::Debug),
        _ => logger.level_for("minions", log::LevelFilter::Trace),
    };
    let logger_colors = fern::colors::ColoredLevelConfig::default();
    logger.format(move |out, message, record| {
               out.finish(format_args!("{}[{}][{}] {}",
                                       chrono::Local::now().format("[%H:%M:%S]"),
                                       logger_colors.color(record.level()),
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

    gtk::init().expect("Failed to initialize GTK");
    let _ = MinionsApp::new(&configfile);
    gtk::main();
}
