/*
* @Author: BlahGeek
* @Date:   2017-04-19
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-05-13
*/

use std;
use std::fmt;
use mcore::action::{Action, ActionArg};

/// Typed data in item
#[derive(Debug)]
pub enum ItemData {
    Text(String),
    Path(std::path::PathBuf),
}

/// The item type (represents single selectable item (row))
pub struct Item {
    /// Main title text
    pub title: String,
    /// Sub-title text
    pub subtitle: Option<String>,
    // TODO: icon
    /// Badge text (like label), optional
    pub badge: Option<String>,

    /// Item data, for quick-send and/or info
    pub data: Option<ItemData>,

    /// Search str, fallback to title
    pub search_str: Option<String>,

    /// Action, optional
    pub action: Option<Box<Action>>,
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
        if let & Some(ref action) = &self.action {
            try!(write!(f, "[ACTION: {}, {:?}]", action.name(), self.action_arg));
        };
        Ok(())
    }
}


impl Item {

    pub fn new() -> Item {
        Item {
            title: String::new(),
            subtitle: None,
            badge: None,
            data: None,
            search_str: None,
            action: None,
            action_arg: ActionArg::None,
        }
    }

    pub fn new_text_item(text: &str) -> Item {
        let mut item = Item::new();
        item.title = text.into();
        item.data = Some(ItemData::Text(text.into()));
        item
    }

    pub fn new_action_item(action: Box<Action>) -> Item {
        let mut item = Item::new();
        item.title = action.name().into();
        item.action = Some(action);
        item
    }

}
