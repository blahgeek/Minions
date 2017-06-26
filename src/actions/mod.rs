/*
* @Author: BlahGeek
* @Date:   2017-04-18
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-26
*/

mod utils;

mod capital;
mod linux_desktop_entry;
mod search_engine;
mod file_browser;
mod custom_script;
mod youdao;

use toml;

use std::fmt;
use std::sync::Arc;
use std::error::Error;
use std::path::{PathBuf, Path};

#[derive(Clone)]
#[derive(Debug)]
enum ActionError {
    FileFormatError(PathBuf),
    ServiceError(String),
    NotSupported,
    Unknown,
}

impl Error for ActionError {
    fn description(&self) -> &str {
        "Error generation action"
    }
}
impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

use mcore::action::Action;

pub fn get_actions(config: toml::Value) -> Vec<Arc<Box<Action>>> {
    let mut ret : Vec<Arc<Box<Action>>> = vec![
        Arc::new(Box::new(capital::Capital{})),
    ];
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
    if let Some(opts) = config.get("plugin_directories") {
        if let Some(opts) = opts.as_array() {
            for dir in opts {
                if let Some(dir) = dir.as_str() {
                    for x in custom_script::ScriptAction::get_all(Path::new(dir)) {
                        ret.push(Arc::new(Box::new(x)));
                    }
                }
            }
        }
    }
    ret.push(Arc::new(Box::new(youdao::Youdao{})));

    ret
}
