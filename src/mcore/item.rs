/*
* @Author: BlahGeek
* @Date:   2017-04-19
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-08-06
*/

use std;
use std::fmt;
use std::sync::Arc;
use mcore::action::{Action, ActionArg};

#[derive(Debug, Clone)]
pub enum Icon {
    GtkName(String),
    Character{ch: char, font: String},
    File(std::path::PathBuf),
}

/// Typed data in item
#[derive(Debug, Clone)]
pub enum ItemData {
    Text(String),
    Path(std::path::PathBuf),
}

/// The item type (represents single selectable item (row))
#[derive(Clone)]
pub struct Item {
    /// Main title text
    pub title: String,
    /// Sub-title text
    pub subtitle: Option<String>,
    /// Icon, optional
    pub icon: Option<Icon>,
    /// Badge text (like label), optional
    pub badge: Option<String>,
    /// Priority, smaller is more important, default to 0
    pub priority: i32,

    /// Item data, for quick-send and/or info
    pub data: Option<ItemData>,

    /// Search str, fallback to title
    pub search_str: Option<String>,

    /// Action, optional
    pub action: Option<Arc<Box<Action + Sync + Send>>>,
    /// Argument for action, optional
    pub action_arg: ActionArg,
}


impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}", self.title));
        if let & Some(ref subtitle) = &self.subtitle {
            try!(write!(f, "({})", subtitle));
        };
        if let & Some(ref badge) = &self.badge {
            try!(write!(f, "[{}]", badge));
        };
        Ok(())
    }
}


impl Item {

    pub fn new(title: &str) -> Item {
        Item {
            title: title.into(),
            subtitle: None,
            icon: None,
            badge: None,
            priority: 0,
            data: None,
            search_str: None,
            action: None,
            action_arg: ActionArg::None,
        }
    }

    pub fn new_path_item(path: &std::path::Path) -> Item {
        let mut item = Item::new(&path.to_string_lossy());
        item.data = Some(ItemData::Path(path.into()));
        item.icon = Some(Icon::Character{ch: '', font: "FontAwsome".into()});
        item
    }

    pub fn new_text_item(text: &str) -> Item {
        let mut item = Item::new(text);
        item.data = Some(ItemData::Text(text.into()));
        item
    }

    pub fn new_action_item(action: Arc<Box<Action + Sync + Send>>) -> Item {
        let mut item = action.get_item();
        item.action = Some(action);
        item
    }

    /// Get searchable str reference
    /// aka `search_str` or `title`
    pub fn get_search_str(&self) -> &str {
        if let Some(ref search_str) = self.search_str {
            &search_str
        } else {
            &self.title
        }
    }

    /// Get data for copy
    pub fn get_copy_str(&self) -> &str {
        match self.data {
            Some(ItemData::Text(ref text)) => text,
            Some(ItemData::Path(ref path)) => &path.to_str().unwrap(),
            _ => &self.title,
        }
    }

}
