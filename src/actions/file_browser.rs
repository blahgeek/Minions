/*
* @Author: BlahGeek
* @Date:   2017-06-17
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-26
*/

use toml;

use std::sync::Arc;
use std::path::{PathBuf, Path};
use std::error::Error;

use mcore::item::{Item, ItemData};
use mcore::action::Action;
use actions::utils::open;

pub struct FileBrowserEntry {
    name: String,
    path: PathBuf,
    is_file: bool,
}


#[derive(Deserialize)]
struct EntryConfig {
    name: String,
    path: String,
}

#[derive(Deserialize)]
struct Config {
    entries: Vec<EntryConfig>,
}

impl FileBrowserEntry {
    pub fn new(name: String, path: PathBuf) -> Option<FileBrowserEntry> {
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

    pub fn get_all(config: toml::Value) -> Vec<FileBrowserEntry> {
        let config = config.try_into::<Config>();
        if let Err(ref error) = config {
            warn!("Error loading file browser entry config: {}", error);
            return Vec::new();
        }
        let config = config.unwrap();

        config.entries.into_iter()
        .map(|c| {
            FileBrowserEntry::new(c.name, Path::new(&c.path).to_path_buf())
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .collect()
    }
}


impl Action for FileBrowserEntry {
    fn get_item (&self) -> Item {
        let mut ret = Item::new(&self.name);
        ret.subtitle = Some(self.path.to_string_lossy().into());
        ret.badge = if self.is_file {
            Some("File".into())
        } else {
            Some("Directory".into())
        };
        ret.data = Some(ItemData::Path(self.path.clone()));
        ret.priority = -10;
        ret
    }

    fn accept_nothing(&self) -> bool { true }

    fn should_return_items(&self) -> bool { !self.is_file }

    fn run(&self) -> Result<Vec<Item>, Box<Error>> {
        if self.is_file {
            info!("open: {:?}", self.path);
            open::that(&self.path)?;
            Ok(Vec::new())
        } else {
            let mut ret = Vec::new();

            debug!("Reading dir: {:?}", self.path);
            let entries = self.path.read_dir()?;
            for entry in entries.into_iter() {
                match entry {
                    Ok(entry) => {
                        if let Some(act) = FileBrowserEntry::new(entry.file_name().to_string_lossy().into(), entry.path()) {
                            let mut item = act.get_item();
                            item.action = Some(Arc::new(Box::new(act)));
                            ret.push(item);
                        }
                    },
                    Err(error) => {
                        warn!("Read dir error: {}", error);
                    }
                }
            }
            if let Some(parent) = self.path.parent() {
                if let Some(act) = FileBrowserEntry::new("..".into(), parent.into()) {
                    let mut item = act.get_item();
                    item.action = Some(Arc::new(Box::new(act)));
                    item.priority = -100;
                    ret.push(item);
                }
            }
            Ok(ret)
        }
    }
}
