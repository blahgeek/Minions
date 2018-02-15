mod parser;
mod item;
mod action;
mod requirement;

use toml;
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;

use mcore::item::Item;
use mcore::config::Config;

use actions::ActionError;
use self::item::ScriptItem;

fn get_item(script_dir: &Path) -> Result<Item, Box<Error>> {

    let itemfile = script_dir.join("item.toml");
    debug!("Reading script item: {:?}", itemfile);

    let mut itemdata = String::new();
    if let Ok(mut itemfile) = File::open(&itemfile) {
        itemfile.read_to_string(&mut itemdata)?;
    }
    let mut item : ScriptItem = toml::from_str(&itemdata)?;

    if item.title.len() == 0 {
        return Err(Box::new(ActionError::new("Invalid item.toml")));
    }

    if item.badge.is_none() {
        item.badge = Some("Script".into());
    }

    for req_text in item.requirements.iter() {
        if let Some(req) = requirement::Requirement::new(&req_text) {
            if !req.check() {
                info!("Requirement {:?} for plugin {} not met, cannot load", req, item.title);
                return Err(Box::new(ActionError::new("Requirements not met")));
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
        if let Err(error) = entries {
            info!("Unable to read dir {:?}: {}", plugin_dir, error);
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
                        warn!("Unable to load custom script at {:?}: {}", entry_path, error);
                    }
                }
            }
        }
    }
    ret

}
