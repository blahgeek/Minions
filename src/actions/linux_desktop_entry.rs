/*
* @Author: BlahGeek
* @Date:   2017-05-01
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-05-14
*/

extern crate shlex;

extern crate ini;
use self::ini::Ini;

use std::ffi::OsStr;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use mcore::action::{Action, Icon};
use mcore::item::Item;
use actions::ActionError;

#[derive(Debug)]
pub struct LinuxDesktopEntry {
    name: String,
    comment: Option<String>,
    exec: Vec<String>,
    icon_text: Option<String>,

    // TODO: exec, accept_path, accept_url
}

impl Action for LinuxDesktopEntry {
    fn name(&self) -> &str { &self.name }
    fn icon(&self) -> Option<Icon> {
        if let Some(ref icon_text) = self.icon_text {
            if icon_text.chars().next() == Some('/') {
                Some(Icon::File(PathBuf::from(&icon_text)))
            } else {
                Some(Icon::Name(icon_text.clone()))
            }
        } else {
            None
        }
    }

    fn accept_nothing(&self) -> bool { true }

    fn accept_path(&self) -> bool {
        self.exec.iter().find(|arg| (*arg == "%f" || *arg == "%f")).is_some()
    }

    fn run_path(&self, path: &Path) -> Result<Vec<Item>, Box<Error>> {
        self.run_path_or_empty(Some(path))
    }

    fn run(&self) -> Result<Vec<Item>, Box<Error>> {
        self.run_path_or_empty(None)
    }

    fn should_return_items(&self) -> bool { false }
}

impl LinuxDesktopEntry {

    fn run_path_or_empty(&self, path: Option<&Path>) -> Result<Vec<Item>, Box<Error>> {
        if self.exec.len() <= 1 {
            return Err(Box::new(ActionError::NotSupported));
        }
        let mut cmd = Command::new(&self.exec[0]);
        for arg in self.exec.iter().skip(1) {
            if *arg == "%f" || *arg == "%F" {
                if let Some(p) = path {
                    cmd.arg(p.as_os_str());
                }
            } else if *arg == "%u" || *arg == "%F" {
                // nop
            } else {
                cmd.arg(arg);
            }
        }
        cmd.spawn()?;
        Ok(Vec::new())
    }


    // TODO: translate, icon, path, is_terminal
    fn get(filepath: &Path) -> Result<LinuxDesktopEntry, Box<Error>> {
        let config = Ini::load_from_file(filepath)?;
        let typ = config.get_from_or(Some("Desktop Entry"), "Type", "");
        if typ != "Application" {
            return Err(Box::new(ActionError::NotSupported));
        }

        let err = ActionError::FileFormatError(filepath.into());

        let exec_str = config.get_from(Some("Desktop Entry"), "Exec").ok_or(err.clone())?;

        Ok(LinuxDesktopEntry {
            name: config.get_from(Some("Desktop Entry"), "Name").ok_or(err.clone())?.into(),
            comment: Some(config.get_from(Some("Desktop Entry"), "Comment").ok_or(err.clone())?.into()),
            exec: shlex::split(exec_str).ok_or(err.clone())?,
            icon_text: match config.get_from(Some("Desktop Entry"), "Icon") {
                Some(s) => Some(s.into()),
                None => None,
            },
        })
    }

    pub fn get_all() -> Vec<LinuxDesktopEntry> {
        let application_dirs = [
            Path::new("/usr/local/share/applications/"),
            Path::new("/usr/share/applications/"),
        ];
        let mut ret = Vec::new();

        for application_dir in application_dirs.iter() {
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

#[cfg(test)]
mod tests {
    use mcore::action::Action;
    use std::path::PathBuf;
    use actions::linux_desktop_entry::LinuxDesktopEntry;

    #[test]
    fn linux_desktop_entry_single_test() {
        let path = PathBuf::from("/usr/share/applications/atom.desktop");
        let action = LinuxDesktopEntry::get(&path).unwrap();
        assert!(action.name() == "Atom");
    }

    #[test]
    fn linux_desktop_entry_test() {
        let items = LinuxDesktopEntry::get_all();
        for item in items.into_iter() {
            println!("{:?}", item);
        }
    }

}
