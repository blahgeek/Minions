/*
* @Author: BlahGeek
* @Date:   2017-07-20
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-02-08
*/

extern crate uuid;
use self::uuid::Uuid;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use mcore::action::{Action, ActionResult, ActionArg};
use mcore::item::{Item, Icon};

pub struct SaveTxtAction {
    tempdir: PathBuf,
}


impl Action for SaveTxtAction {
    fn get_item(&self) -> Item {
        Item {
            title: "Save as TXT".into(),
            subtitle: Some(format!("Save text into .txt under {:?}", self.tempdir)),
            icon: Some(Icon::Character{ch: '', font: "FontAwsome".into()}),
            badge: None,
            priority: 0,
            data: None,
            search_str: None,
            action: None,
            action_arg: ActionArg::None,
        }
    }

    fn accept_text(&self) -> bool { true }

    fn run_text(&self, text: &str) -> ActionResult {
        let mut filepath = self.tempdir.clone();
        filepath.push(format!("minions-tmp-{}.txt", Uuid::new_v4().simple().to_string()));
        let mut f = File::create(&filepath)?;
        f.write_all(text.as_bytes())?;

        let mut ret = Item::new_path_item(&filepath);
        ret.subtitle = Some(format!("{} bytes written", text.len()));

        Ok(vec![ret])
    }
}

impl SaveTxtAction {
    pub fn new() -> SaveTxtAction {
        SaveTxtAction {
            tempdir: env::temp_dir()
        }
    }
}
