/*
* @Author: BlahGeek
* @Date:   2017-06-17
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-04-01
*/

use std::env;
use std::sync::Arc;
use std::path::{PathBuf, Path};

use mcore::item::{Item, Icon};
use mcore::action::{Action, ActionResult};
use mcore::config::Config;
use actions::utils::open;

struct FileBrowserEntry {
    name: String,
    path: PathBuf,
    is_file: bool,
}


impl FileBrowserEntry {
    fn new(name: String, path: PathBuf) -> Option<FileBrowserEntry> {
        if ! (path.is_dir() || path.is_file()) {
            warn!("Invalid path: {:?}", path);
            None
        } else {
            let is_file = path.is_file();
            Some(FileBrowserEntry {
                name: name,
                path: path,
                is_file: is_file,
            })
        }
    }

    fn into_item(self) -> Item {
        let mut ret = Item::new(&self.name);
        ret.subtitle = Some(self.path.to_string_lossy().into());
        ret.badge = if self.is_file {
            Some("File".into())
        } else {
            Some("Directory".into())
        };
        ret.icon = Some(if self.is_file {
            Icon::FontAwesome("file".into())
        } else {
            Icon::FontAwesome("folder".into())
        });
        ret.data = Some(self.path.to_string_lossy().into());
        ret.priority = -10;
        ret.action = Some(Arc::new(Box::new(self)));
        ret
    }
}


impl Action for FileBrowserEntry {

    fn runnable_bare (&self) -> bool { true }

    fn run_bare (&self) -> ActionResult {
        if self.is_file {
            open::that(&self.path.to_string_lossy())?;
            Ok(Vec::new())
        } else {
            let mut ret = Vec::new();

            debug!("Reading dir: {:?}", self.path);
            let entries = self.path.read_dir()?;
            for entry in entries.into_iter() {
                match entry {
                    Ok(entry) => {
                        if let Some(act) = FileBrowserEntry::new(entry.file_name().to_string_lossy().into(), entry.path()) {
                            ret.push(act.into_item())
                        }
                    },
                    Err(error) => {
                        warn!("Read dir error: {}", error);
                    }
                }
            }
            if let Some(parent) = self.path.parent() {
                if let Some(act) = FileBrowserEntry::new("..".into(), parent.into()) {
                    let mut item = act.into_item();
                    item.priority = -100;
                    ret.push(item);
                }
            }
            Ok(ret)
        }
    }
}

#[derive(Deserialize)]
struct EntryConfig {
    name: String,
    path: String,
}


pub fn get(config: &Config) -> Vec<Item> {
    let entries = config.get::<Vec<EntryConfig>>(&["file_browser", "entries"]).unwrap();

    entries.into_iter()
        .map(|c| {
            let mut p = Path::new(&c.path).to_path_buf();
            if c.path.starts_with("~/") {
                if let Some(homedir) = env::home_dir() {
                    p = homedir;
                    p.push(Path::new(&c.path[2..]));
                }
            }
            FileBrowserEntry::new(c.name, p)
        })
        .filter(|x| x.is_some())
            .map(|x| x.unwrap().into_item())
            .collect()
}
