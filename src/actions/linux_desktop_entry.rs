/*
* @Author: BlahGeek
* @Date:   2017-05-01
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-02-15
*/

extern crate shlex;

extern crate ini;
use self::ini::Ini;

use std::ffi::OsStr;
use std::error::Error;
use std::sync::Arc;
use std::path::Path;
use mcore::action::{Action, ActionResult};
use mcore::item::{Item, Icon};
use mcore::config::Config;
use actions::ActionError;
use actions::utils::subprocess;

#[derive(Debug)]
struct LinuxDesktopEntry {
    name: String,
    comment: Option<String>,
    exec: Vec<String>,
    icon_text: Option<String>,
    terminal: bool,
}

impl Action for LinuxDesktopEntry {

    fn runnable_bare (&self) -> bool { true }

    fn runnable_arg (&self) -> bool {
        self.exec.iter().find(
            |arg| (*arg == "%f" || *arg == "%F" || *arg == "%u" || *arg == "%U")
        ).is_some()
    }

    fn run_arg (&self, arg: &str) -> ActionResult {
        self.run_path_or_empty(Some(arg))
    }

    fn run_bare (&self) -> ActionResult {
        self.run_path_or_empty(None)
    }
}

impl LinuxDesktopEntry {

    fn run_path_or_empty(&self, path: Option<&str>) -> ActionResult {
        if self.exec.len() <= 0 {
            return Err(Box::new(ActionError::new("Executable path is empty")));
        }

        let mut cmd : Vec<&str> = Vec::new();
        if self.terminal {
            cmd.push("sh");
            cmd.push("-c");
            cmd.push(include_str!("./utils/sensible-terminal.sh"));
            cmd.push("sensible-terminal.sh");
            cmd.push("-e");
        }

        for arg in self.exec.iter() {
            let larg = arg.to_lowercase();
            if larg == "%f" || larg == "%u" {
                if let Some(p) = path {
                    cmd.push(p);
                }
            } else {
                cmd.push(&arg);
            }
        }

        subprocess::spawn(cmd[0], &cmd[1..])?;
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

    fn get_all(config: &Config) -> Vec<LinuxDesktopEntry> {
        let directories = config.get::<Vec<String>>(&["linux_desktop_entry", "directories"]).unwrap();

        let application_dirs = directories.iter().map(|x| Path::new(x));
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


pub fn get(config: &Config) -> Vec<Item> {
    LinuxDesktopEntry::get_all(config).into_iter()
        .map(|action| {
            let exe_path = if action.exec.len() > 0 {
                Some(action.exec[0].clone())
            } else { None };
            let comment = action.comment.clone();

            let mut item = Item::new(&action.name);
            if let Some(exe_path) = exe_path {
                item.data = Some(exe_path);
            }
            item.subtitle = comment;
            item.badge = Some("Desktop Entry".into());

            item.icon = if let Some(ref icon_text) = action.icon_text {
                Some( if icon_text.starts_with("/") {
                    Icon::File(Path::new(&icon_text).to_path_buf())
                } else {
                    Icon::GtkName(icon_text.clone())
                })
            } else {
                Some(Icon::GtkName("gtk-missing-image".into()))
            };
            item.action = Some(Arc::new(Box::new(action)));
            item
        }).collect()
}

