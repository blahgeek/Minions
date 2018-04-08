/*
* @Author: BlahGeek
* @Date:   2017-07-16
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-04-08
*/

extern crate gtk;
extern crate glib;
extern crate gdk;
extern crate gtk_sys;
extern crate libc;
extern crate chrono;

use self::glib::signal::connect;
use self::glib::translate::*;
use self::gtk::Clipboard;
use self::gtk::ClipboardExt;

use std::sync::Arc;
use std::mem::transmute;

use mcore::action::{Action, ActionResult};
use mcore::item::{Item, Icon};
use mcore::config::Config;
use mcore::lrudb::LruDB;
use mcore::errors::*;

unsafe extern "C" fn trampoline(clipboard: *mut gtk_sys::GtkClipboard,
                                _: *mut libc::c_void,
                                f: &Box<Fn(&Clipboard) + 'static>) {
    f(&Clipboard::from_glib_none(clipboard))
}


fn connect_clipboard_change<F>(clipboard: &Clipboard, f: F)
where F: Fn(&Clipboard) + 'static {
    unsafe {
        let f: Box<Box<Fn(&Clipboard) + 'static>> =
            Box::new(Box::new(f));
        connect(clipboard.to_glib_none().0, "owner-change",
                transmute(trampoline as usize), Box::into_raw(f) as *mut _);
    }
}


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
                let mut item = Item::new_text_item(&x.data);
                item.subtitle = Some(format!("{}, {} bytes",
                                             x.time.format("%T %b %e").to_string(),
                                             x.data.len()));
                item.icon = Some(Icon::FontAwesome("paste".into()));
                item
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
        connect_clipboard_change(&clipboard, move |clipboard| {
            let content = clipboard.wait_for_text();
            if let Some(text) = content {
                trace!("New clipboard text: {:?}", text);
                if ignore_single_byte && text.len() <= 1 {
                    debug!("Single byte, do not store in history");
                } else if let Err(err) = lrudb.add("clipboard_history", &text, history_max_len as i32) {
                    warn!("Unable to store clipboard text: {}", err);
                }
            }
        });
        action
    }
}

pub fn get(config: &Config) -> Item {
    let action = ClipboardHistoryAction::new(config);
    let mut item = Item::new("Clipboard History");
    item.subtitle = Some(format!("View clipboard history up to {} entries",
                                 action.history_max_len));
    item.icon = Some(Icon::FontAwesome("paste".into()));
    item.action = Some(Arc::new(action));
    item
}
