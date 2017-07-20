/*
* @Author: BlahGeek
* @Date:   2017-04-18
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-07-20
*/

mod utils;

mod linux_desktop_entry;
mod search_engine;
mod file_browser;
mod custom_script;
mod save_txt;
mod youdao;

#[cfg(feature="use-gtk")]
mod clipboard;

use toml;

use std::fmt;
use std::sync::Arc;
use std::error::Error;
use std::path::{PathBuf, Path};

#[derive(Clone)]
#[derive(Debug)]
struct ActionError {
    reason: String,
}

impl ActionError {
    fn new(reason: &str) -> ActionError {
        ActionError { reason: reason.into() }
    }
}

impl Error for ActionError {
    fn description(&self) -> &str {
        &self.reason
    }
}
impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

use mcore::action::Action;

pub fn get_actions(config: toml::Value) -> Vec<Arc<Box<Action + Sync + Send>>> {
    let mut ret : Vec<Arc<Box<Action + Sync + Send>>> = vec![];
    if let Some(opts) = config.get("linux_desktop_entry") {
        for desktop_entry in linux_desktop_entry::LinuxDesktopEntry::get_all(opts.clone()) {
            ret.push(Arc::new(Box::new(desktop_entry)));
        }
    }
    if let Some(opts) = config.get("search_engine") {
        for se in search_engine::SearchEngine::get_all(opts.clone()) {
            ret.push(Arc::new(Box::new(se)));
        }
    }
    if let Some(opts) = config.get("file_browser") {
        for x in file_browser::FileBrowserEntry::get_all(opts.clone()) {
            ret.push(Arc::new(Box::new(x)));
        }
    }

    ret.push(Arc::new(Box::new(youdao::Youdao{})));
    ret.push(Arc::new(Box::new(save_txt::SaveTxtAction::new())));

    if cfg!(feature="use-gtk") {
        if let Some(opts) = config.get("clipboard_history") {
            if let Some(max_len) = opts["max_entries"].as_integer() {
                let action = clipboard::ClipboardHistoryAction::new(max_len as usize);
                ret.push(Arc::new(Box::new(action)));
            }
        }
    }

    let mut plugin_dirs : Vec<PathBuf> = vec![
        Path::new("./plugins/").to_path_buf(),
        Path::new("./usr/share/minions-plugins/").to_path_buf(),
        Path::new("/usr/share/minions-plugins/").to_path_buf(),
    ];
    if let Some(opts) = config.get("extra_plugin_directories") {
        if let Some(opts) = opts.as_array() {
            for dir in opts {
                if let Some(dir) = dir.as_str() {
                    plugin_dirs.push(Path::new(dir).to_path_buf());
                }
            }
        }
    }
    plugin_dirs.dedup();

    if plugin_dirs.len() == 0 {
        warn!("No plugin directory defined");
    }

    for plugin_dir in plugin_dirs.iter() {
        info!("Loading plugins from {:?}", plugin_dir);
        for x in custom_script::ScriptAction::get_all(&plugin_dir) {
            ret.push(Arc::new(Box::new(x)));
        }
    }

    ret
}
