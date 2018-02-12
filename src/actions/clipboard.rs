/*
* @Author: BlahGeek
* @Date:   2017-07-16
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-02-12
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
use self::chrono::{Local, DateTime};

use std::sync::{Arc, Mutex};
use std::mem::transmute;
use std::collections::VecDeque;

use actions::ActionError;
use mcore::action::{Action, ActionResult};
use mcore::item::{Item, Icon};
use mcore::config::Config;

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
    history: Arc<Mutex<VecDeque<(String, DateTime<Local>)>>>,
}

impl Action for ClipboardHistoryAction {

    fn runnable_bare(&self) -> bool { true }

    fn run_bare(&self) -> ActionResult {
        if let Ok(history) = self.history.lock() {
            debug!("Returning {} clipboard histories", history.len());
            if history.len() == 0 {
                Err(Box::new(ActionError::new("No clipboard history available")))
            } else {
                Ok(history.iter().map(|x| {
                    let mut item = Item::new_text_item(&x.0);
                    item.subtitle = Some(format!("{}, {} bytes",
                                                 x.1.format("%T %b %e").to_string(),
                                                 x.0.len()));
                    item.icon = Some(Icon::Character{ch: '', font: "FontAwesome".into()});
                    item
                }).collect())
            }
        } else {
            Err(Box::new(ActionError::new("Unable to unlock history")))
        }
    }
}

impl ClipboardHistoryAction {
    fn new(config: &Config) -> ClipboardHistoryAction {
        let history_max_len = config.get::<usize>(&["clipboard_history", "max_entries"]).unwrap();
        let ignore_single_byte = config.get::<bool>(&["clipboard_history", "ignore_single_byte"]).unwrap();

        let action = ClipboardHistoryAction {
            history_max_len: history_max_len,
            history: Arc::new(Mutex::new(VecDeque::new())),
        };
        let history = action.history.clone();

        let clipboard = gtk::Clipboard::get(&gdk::Atom::intern("CLIPBOARD"));
        connect_clipboard_change(&clipboard, move |clipboard| {
            let content = clipboard.wait_for_text();
            if let Some(text) = content {
                trace!("New clipboard text: {:?}", text);
                if let Ok(mut history) = history.lock() {
                    let is_dup = if let Some(front) = history.front() {
                        text == front.0.as_str()
                    } else {
                        false
                    };
                    if is_dup {
                        debug!("Duplicate, do not push to history");
                    } else if ignore_single_byte && text.len() <= 1 {
                        debug!("Single byte, do not push to history");
                    } else {
                        history.push_front((text.into(), Local::now()));
                    }
                    while history.len() > history_max_len {
                        history.pop_back().unwrap();
                    }
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
    item.icon = Some(Icon::Character{ch: '', font: "FontAwesome".into()});
    item.action = Some(Arc::new(Box::new(action)));
    item
}
