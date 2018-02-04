/*
* @Author: BlahGeek
* @Date:   2017-04-18
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-02-04
*/

mod utils;

mod linux_desktop_entry;
mod search_engine;
mod file_browser;
mod custom_script;
mod save_txt;
mod youdao;
mod clipboard;

use std::fmt;
use std::sync::Arc;
use std::error::Error;
use std::path::{PathBuf, Path};

use mcore::config::Config;

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

pub fn get_actions(config: &Config) -> Vec<Arc<Box<Action + Sync + Send>>> {
    let mut ret : Vec<Arc<Box<Action + Sync + Send>>> = vec![];

    for desktop_entry in linux_desktop_entry::LinuxDesktopEntry::get_all(config) {
        ret.push(Arc::new(Box::new(desktop_entry)));
    }
    for se in search_engine::SearchEngine::get_all(config) {
        ret.push(Arc::new(Box::new(se)));
    }
    for x in file_browser::FileBrowserEntry::get_all(config) {
        ret.push(Arc::new(Box::new(x)));
    }

    ret.push(Arc::new(Box::new(youdao::Youdao{})));
    ret.push(Arc::new(Box::new(save_txt::SaveTxtAction::new())));
    ret.push(Arc::new(Box::new(clipboard::ClipboardHistoryAction::new(config))));

    let mut plugin_dirs : Vec<PathBuf> = vec![
        Path::new("./plugins/").to_path_buf(),
        Path::new("./usr/share/minions-plugins/").to_path_buf(),
        Path::new("/usr/share/minions-plugins/").to_path_buf(),
    ];
    for dir in config.get::<Vec<String>>(&["core", "extra_plugin_directories"]).unwrap().iter() {
        plugin_dirs.push(Path::new(&dir).to_path_buf());
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
