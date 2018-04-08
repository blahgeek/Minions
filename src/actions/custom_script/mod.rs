pub mod parser;
mod item;
mod action;
mod requirement;

use toml;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;

use mcore::item::Item;
use mcore::config::Config;
use mcore::errors::*;
use error_chain::ChainedError;

use self::item::ScriptItem;

fn get_item(script_dir: &Path) -> Result<Item> {

    let itemfile = script_dir.join("item.toml");
    debug!("Reading script item: {:?}", itemfile);

    let mut itemdata = String::new();
    if let Ok(mut itemfile) = File::open(&itemfile) {
        itemfile.read_to_string(&mut itemdata)?;
    }
    let mut item : ScriptItem = toml::from_str(&itemdata)
        .map_err(|e| Error::with_chain(e, "Failed parsing item.toml"))?;

    if item.title.len() == 0 {
        bail!("Invalid item.toml: empty title");
    }

    if item.badge.is_none() {
        item.badge = Some("Script".into());
    }

    for req_text in item.requirements.iter() {
        if let Some(req) = requirement::Requirement::new(&req_text) {
            if !req.check() {
                bail!("Requirement not met: {:?}", req);
            }
        } else {
            warn!("Invalid requirement string {}, ignore", req_text);
        }
    }

    Ok(item.into_item(script_dir))
}

pub fn get(config: &Config) -> Vec<Item> {

    let mut plugin_dirs : Vec<PathBuf> = vec![
        Path::new("./plugins/").to_path_buf(),
        Path::new("./usr/share/minions-plugins/").to_path_buf(),
        Path::new("/usr/share/minions-plugins/").to_path_buf(),
    ];
    for dir in config.get::<Vec<String>>(&["core", "extra_plugin_directories"]).unwrap().iter() {
        plugin_dirs.push(Path::new(&dir).to_path_buf());
    }
    plugin_dirs.dedup();

    let mut ret : Vec<Item> = Vec::new();

    for plugin_dir in plugin_dirs.iter() {
        info!("Loading custom action from {:?}", plugin_dir);

        let entries = plugin_dir.read_dir();
        if let Err(_) = entries {
            info!("Unable to read dir {:?}, ignore", plugin_dir);
            continue;
        }
        let entries = entries.unwrap();

        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path : PathBuf = entry.path();
                match get_item(&entry_path) {
                    Ok(x) => {
                        info!("Loaded custom script at {:?}", entry_path);
                        ret.push(x);
                    },
                    Err(error) => {
                        warn!("Unable to load custom script at {:?}: {}",
                              entry_path, error.display_chain());
                    }
                }
            }
        }
    }
    ret

}
