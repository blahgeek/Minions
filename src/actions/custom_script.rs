/*
* @Author: BlahGeek
* @Date:   2017-06-18
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-18
*/

/// Action defined by custom script

extern crate open;
extern crate serde_json;

use toml;

use std;
use std::rc::Rc;
use std::path::PathBuf;
use std::error::Error;
use std::process;
use std::process::Command;

use std::fs::File;
use std::io::prelude::*;

use mcore::item::{Item, ItemData};
use mcore::action::{Action, ActionArg};
use actions::file_browser::FileBrowserEntry;

/// Output item from custom script
/// Each item consists of:
///     title
///     subtitle (optional)
///     badge (optional)
/// Each item may have one of following data included:
///     data_text, data_path, data_url
/// And each item may define it's action, that may be one of the followings:
///     action_callback: call custom script again, with `action_callback` as command and arguments
///                      must also define action_callback_returns: whether this action would return items
///     action_children: pre-define its children items
///   if neither action_callback or action_children is defined,
///   the action would automatically defined according to data included (if any)

#[derive(Deserialize, Clone)]
struct ScriptItem {
    title: String,
    subtitle: Option<String>,
    badge: Option<String>,

    data_text: Option<String>,
    data_path: Option<String>,
    data_url: Option<String>,

    action_callback: Option<Vec<String>>,
    action_callback_returns: Option<bool>,
    action_children: Option<Vec<ScriptItem>>,
}

#[derive(Deserialize)]
struct ScriptOutput {
    results: Vec<ScriptItem>,
}

// some actions used by ScriptItem
struct OpenURLAction {
    url: String,
}

struct PredefinedChildrenAction {
    script_dir: PathBuf,
    children: Vec<ScriptItem>,
}

pub struct ScriptAction {
    name: String,
    description: String,

    script_dir: PathBuf,

    accept_nothing_: bool,
    accept_text_: bool,
    accept_path_: bool,

    script: String,
    script_args: Vec<String>,
    script_returns: bool,
}

impl Action for OpenURLAction {
    fn get_item (&self) -> Item { Item::new("unimplemented") } // unused
    fn accept_nothing(&self) -> bool { true }
    fn should_return_items(&self) -> bool { false }
    fn run(&self) -> Result<Vec<Item>, Box<Error>> {
        open::that(&self.url)?;
        Ok(Vec::new())
    }
}

impl Action for PredefinedChildrenAction {
    fn get_item (&self) -> Item { Item::new("unimplemented") } // unused
    fn accept_nothing(&self) -> bool { true }
    fn should_return_items(&self) -> bool { self.children.len() > 0 }
    fn run(&self) -> Result<Vec<Item>, Box<Error>> {
        Ok(self.children.iter()
           .map(|x| x.clone().into_item(&self.script_dir))
           .collect())
    }
}

fn output_to_items(output: &[u8], script_dir: &std::path::Path) -> Result<Vec<Item>, Box<Error>> {
    let json_output : ScriptOutput = serde_json::from_slice(output)?;
    Ok(json_output.results.into_iter()
       .map(|x| x.into_item(script_dir))
       .collect())
}

impl Action for ScriptAction {
    fn get_item (&self) -> Item {
        let mut item = Item::new(&self.name);
        item.subtitle = Some(self.description.clone());
        item.badge = Some("Script".into());
        item.priority = -50;
        item
    }
    fn accept_nothing(&self) -> bool { self.accept_nothing_ }
    fn accept_text(&self) -> bool { self.accept_text_ }
    fn accept_path(&self) -> bool { self.accept_path_ }
    fn should_return_items(&self) -> bool { self.script_returns }
    fn run(&self) -> Result<Vec<Item>, Box<Error>> {
        let mut cmd = Command::new(&self.script_dir.join(&self.script));
        cmd.args(&self.script_args);
        debug!("Running script action: {:?}", cmd);
        output_to_items(&cmd.output()?.stdout, &self.script_dir)
    }
    fn run_text(&self, text: &str) -> Result<Vec<Item>, Box<Error>> {
        let mut cmd = Command::new(&self.script_dir.join(&self.script));
        cmd.arg(text);
        debug!("Running script action (with text): {:?}", cmd);
        output_to_items(&cmd.output()?.stdout, &self.script_dir)
    }
    fn run_path(&self, p: &std::path::Path) -> Result<Vec<Item>, Box<Error>> {
        let mut cmd = Command::new(&self.script_dir.join(&self.script));
        cmd.arg(p);
        debug!("Running script action (with path): {:?}", cmd);
        output_to_items(&cmd.output()?.stdout, &self.script_dir)
    }
}

impl ScriptItem {
    fn into_item(self, script_dir: &std::path::Path) -> Item {
        let itemdata = if let Some(text) = self.data_text {
            Some(ItemData::Text(text))
        } else if let Some(ref path) = self.data_path {
            Some(ItemData::Path(std::path::Path::new(path).to_path_buf()))
        } else if let Some(ref url) = self.data_url {
            Some(ItemData::Text(url.clone()))
        } else {
            Some(ItemData::Text(self.title.clone()))
        } ;

        let mut action: Option<Box<Action>> =
        if let Some(action_callback) = self.action_callback {
            if action_callback.len() >= 1 {
                Some(Box::new(ScriptAction {
                    name: "unimplemented".into(), // unused
                    description: "unimplemented".into(), //unused,
                    script_dir: script_dir.to_path_buf(),
                    accept_nothing_: true,
                    accept_text_: false,
                    accept_path_: false,
                    script: action_callback[0].clone(),
                    script_args: action_callback.into_iter().skip(1).collect(),
                    script_returns: self.action_callback_returns.unwrap_or(false),
                }))
            } else {
                warn!("Invalid action_callback in ScriptItem");
                None
            }
        } else if let Some(action_children) = self.action_children {
            Some(Box::new(PredefinedChildrenAction {
                script_dir: script_dir.to_path_buf(),
                children: action_children,
            }))
        } else { None };

        if action.is_none() {
            action = if let Some(url) = self.data_url {
                Some(Box::new(OpenURLAction{ url: url }))
            } else if let Some(path) = self.data_path {
                match FileBrowserEntry::new("unimplemented".into(), std::path::Path::new(&path).to_path_buf()) {
                    Some(x) => Some(Box::new(x)),
                    None => {
                        warn!("Invalid data_path in ScriptItem");
                        None
                    },
                }
            } else { None };
        }

        Item {
            title: self.title,
            subtitle: self.subtitle,
            icon: None,
            badge: self.badge,
            priority: 0,
            data: itemdata,
            search_str: None,
            action: match action {
                Some(x) => Some(Rc::new(x)),
                None => None,
            },
            action_arg: ActionArg::None,
        }
    }
}

#[derive(Deserialize)]
struct ScriptMetadata {
    name: String,
    description: String,

    script: String,
    script_returns: bool,

    accept_nothing: bool,
    accept_text: bool,
    accept_path: bool,
}


impl ScriptAction {
    fn new_from_script_dir(script_dir: &std::path::Path) -> Result<ScriptAction, Box<Error>> {
        let metafile = script_dir.join("metadata.toml");
        debug!("Reading script metadata: {:?}", metafile);

        let mut metadata = String::new();
        if let Ok(mut metafile) = File::open(&metafile) {
            metafile.read_to_string(&mut metadata)?;
        }
        let metadata : ScriptMetadata = toml::from_str(&metadata)?;

        Ok (ScriptAction {
            name: metadata.name,
            description: metadata.description,
            script_dir: script_dir.to_path_buf(),
            accept_nothing_: metadata.accept_nothing,
            accept_text_: metadata.accept_text,
            accept_path_: metadata.accept_path,
            script: metadata.script,
            script_args: Vec::new(),
            script_returns: metadata.script_returns,
        })
    }

    pub fn get_all(scripts_dir: &std::path::Path) -> Vec<ScriptAction> {
        debug!("Loading custom action from {:?}", scripts_dir);

        let entries = scripts_dir.read_dir();
        if let Err(error) = entries {
            warn!("Unable to read dir: {}", error);
            return Vec::new()
        }
        let entries = entries.unwrap();

        let mut ret = Vec::new();

        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                match ScriptAction::new_from_script_dir(&entry_path) {
                    Ok(x) => {
                        debug!("Loaded custom script at {:?}", entry_path);
                        ret.push(x);
                    },
                    Err(error) => {
                        warn!("Unable to load custom script at {:?}: {}", entry_path, error);
                    }
                }
            }
        }
        ret
    }
}
