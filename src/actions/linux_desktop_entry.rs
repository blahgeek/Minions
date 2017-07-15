/*
* @Author: BlahGeek
* @Date:   2017-05-01
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-07-15
*/

extern crate shlex;

extern crate ini;
use self::ini::Ini;

use toml;

use std::ffi::OsStr;
use std::error::Error;
use std::path::{Path, PathBuf};
use mcore::action::{Action, ActionResult};
use mcore::item::{Item, ItemData, Icon};
use actions::ActionError;
use actions::utils::subprocess;

#[derive(Debug)]
pub struct LinuxDesktopEntry {
    name: String,
    comment: Option<String>,
    exec: Vec<String>,
    icon_text: Option<String>,
    terminal: bool,
}

impl Action for LinuxDesktopEntry {

    fn get_item(&self) -> Item {
        let exe_path = if self.exec.len() > 0 {
            Some(self.exec[0].clone())
        } else { None };
        let comment = self.comment.clone();

        let mut item = Item::new(&self.name);
        if let Some(exe_path) = exe_path {
            item.data = Some(ItemData::Path(PathBuf::from(&exe_path)));
        }
        item.subtitle = comment;
        item.badge = Some("Desktop Entry".into());

        if let Some(ref icon_text) = self.icon_text {
            item.icon = Some( if icon_text.starts_with("/") {
                Icon::File(Path::new(&icon_text).to_path_buf())
            } else {
                Icon::GtkName(icon_text.clone())
            })
        }
        item
    }

    fn accept_nothing(&self) -> bool { true }

    fn accept_path(&self) -> bool {
        self.exec.iter().find(|arg| (*arg == "%f" || *arg == "%F")).is_some()
    }

    fn run_path(&self, path: &Path) -> ActionResult {
        self.run_path_or_empty(Some(path))
    }

    fn run(&self) -> ActionResult {
        self.run_path_or_empty(None)
    }

    fn should_return_items(&self) -> bool { false }
}

#[derive(Deserialize)]
struct Config {
    directories: Vec<String>,
}

impl LinuxDesktopEntry {

    fn run_path_or_empty(&self, path: Option<&Path>) -> ActionResult {
        if self.exec.len() <= 0 {
            return Err(Box::new(ActionError::new("Executable path is empty")));
        }

        let mut cmd : Vec<String> = Vec::new();
        if self.terminal {
            cmd.push("sh".into());
            cmd.push("-c".into());
            cmd.push(include_str!("./utils/sensible-terminal.sh").into());
            cmd.push("sensible-terminal.sh".into());
            cmd.push("-e".into());
        }

        cmd.push(self.exec[0].clone());

        for arg in self.exec.iter().skip(1) {
            if *arg == "%f" || *arg == "%F" {
                if let Some(p) = path {
                    cmd.push(p.to_string_lossy().into());
                }
            } else if *arg == "%u" || *arg == "%U" {
                // nop
            } else {
                cmd.push(arg.clone());
            }
        }

        subprocess::spawn(&cmd[0], &cmd.iter().skip(1).map(|x| x.as_str()).collect::<Vec<&str>>())?;
        Ok(Vec::new())
    }


    fn get(filepath: &Path) -> Result<LinuxDesktopEntry, Box<Error>> {
        let config = Ini::load_from_file(filepath)?;
        let typ = config.get_from_or(Some("Desktop Entry"), "Type", "");
        if typ != "Application" {
            return Err(Box::new(ActionError::new("Unsupported desktop entry type")));
        }

        let err = ActionError::new("No exec key found in desktop entry");

        let exec_str = config.get_from(Some("Desktop Entry"), "Exec").ok_or(err.clone())?;

        Ok(LinuxDesktopEntry {
            name: config.get_from(Some("Desktop Entry"), "Name").ok_or(err.clone())?.into(),
            comment: Some(config.get_from_or(Some("Desktop Entry"), "Comment", "").into()),
            exec: shlex::split(exec_str).ok_or(err.clone())?,
            icon_text: match config.get_from(Some("Desktop Entry"), "Icon") {
                Some(s) => Some(s.into()),
                None => None,
            },
            terminal: config.get_from_or(Some("Desktop Entry"), "Terminal", "false") == "true",
        })
    }

    pub fn get_all(config: toml::Value) -> Vec<LinuxDesktopEntry> {
        let config = config.try_into::<Config>();
        if let Err(ref error) = config {
            warn!("Error loading linux desktop entry config: {}", error);
            return Vec::new();
        }
        let config = config.unwrap();

        let application_dirs = config.directories.iter().map(|x| Path::new(x));
        let mut ret = Vec::new();

        for application_dir in application_dirs {
            debug!("Loading linux desktop entries in {:?}", application_dir);
            let entries = application_dir.read_dir();
            if entries.is_err() { continue; }
            let entries = entries.unwrap();
            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    if entry_path.extension() != Some(OsStr::new("desktop")) {
                        continue;
                    }
                    match LinuxDesktopEntry::get(&entry_path) {
                        Ok(item) => ret.push(item),
                        Err(_) => (),
                    }
                }
            }
        }

        ret
    }
}

