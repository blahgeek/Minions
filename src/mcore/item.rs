/*
* @Author: BlahGeek
* @Date:   2017-04-19
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-04-10
*/

use std;
use std::fmt;
use std::sync::Arc;
use mcore::action::Action;

#[derive(Debug, Clone)]
pub enum Icon {
    GtkName(String),
    Character{ch: char, font: String},
    File(std::path::PathBuf),
    FontAwesome(String),
}

/// The item type (represents single selectable item (row))
#[derive(Clone, Default)]
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
    pub data: Option<String>,

    /// Search str, fallback to title
    pub search_str: Option<String>,

    /// Action, optional
    pub action: Option<Arc<Action + Sync + Send>>,
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

    /// Get searchable str reference
    /// aka `search_str` or `title`
    pub fn get_search_str(&self) -> &str {
        if let Some(ref search_str) = self.search_str {
            &search_str
        } else {
            &self.title
        }
    }

}
