/*
* @Author: BlahGeek
* @Date:   2017-07-16
* @Last Modified by:   BlahGeek
* @Last Modified time: 2020-01-17
*/

extern crate gtk;
extern crate glib;
extern crate gdk;
extern crate gtk_sys;
extern crate libc;
extern crate chrono;

use self::glib::ObjectExt;

use std::sync::Arc;

use crate::mcore::action::{Action, ActionResult};
use crate::mcore::item::{Item, Icon};
use crate::mcore::config::Config;
use crate::mcore::lrudb::LruDB;
use crate::mcore::errors::*;


struct ClipboardHistoryAction {
    history_max_len: usize,
    lrudb: LruDB,
}

impl Action for ClipboardHistoryAction {

    fn runnable_bare(&self) -> bool { true }

    fn run_bare(&self) -> ActionResult {
        let history = self.lrudb.getall("clipboard_history")
            .map_err(|e| Error::with_chain(e, "Failed to get clipboard history from LRUDB"))?;
        if history.len() == 0 {
            bail!("No clipboard history available");
        } else {
            Ok(history.iter().map(|x| {
                Item {
                    title: x.data.clone(),
                    subtitle: Some(format!("{}, {} bytes",
                                           x.time.format("%T %b %e").to_string(),
                                           x.data.len())),
                    icon: Some(Icon::FontAwesome("paste".into())),
                    .. Item::default()
                }
            }).collect())
        }
    }
}

impl ClipboardHistoryAction {
    fn new(config: &Config) -> ClipboardHistoryAction {
        let history_max_len = config.get::<usize>(&["clipboard_history", "max_entries"]).unwrap();
        let ignore_single_byte = config.get::<bool>(&["clipboard_history", "ignore_single_byte"]).unwrap();
        let db_file = config.get_filename(&["core", "db_file"]).unwrap();

        let action = ClipboardHistoryAction {
            history_max_len: history_max_len,
            lrudb: LruDB::new(Some(&db_file)).unwrap(),
        };

        let lrudb = LruDB::new(Some(&db_file)).unwrap();
        let clipboard = gtk::Clipboard::get(&gdk::Atom::intern("CLIPBOARD"));
        let clipboard_copied = clipboard.clone();
        clipboard.connect_local("owner-change", true, move |_ignore_value| {
            let content = clipboard_copied.wait_for_text();
            if let Some(text) = content {
                trace!("New clipboard text: {:?}", text);
                if ignore_single_byte && text.len() <= 1 {
                    debug!("Single byte, do not store in history");
                } else if let Err(err) = lrudb.add("clipboard_history", &text, history_max_len as i32) {
                    warn!("Unable to store clipboard text: {}", err);
                }
            }
            None
        }).expect("Unable to connect clipboard signal");
        action
    }
}

pub fn get(config: &Config) -> Item {
    let action = ClipboardHistoryAction::new(config);
    Item {
        title: "Clipboard History".into(),
        subtitle: Some(format!("View clipboard history up to {} entries",
                               action.history_max_len)),
        icon: Some(Icon::FontAwesome("paste".into())),
        action: Some(Arc::new(action)),
        .. Item::default()
    }
}
