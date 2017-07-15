/*
* @Author: BlahGeek
* @Date:   2017-04-20
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-07-15
*/

#[cfg(feature="use-gtk")]
extern crate gtk;

#[cfg(feature="use-gtk")]
extern crate gdk;

extern crate uuid;
use self::uuid::Uuid;

use toml;

use std::fmt;
use std::thread;
use std::error::Error;
use std::sync::Arc;
use mcore::action::{Action, ActionArg, ActionResult};
use mcore::item::{Item, ItemData, Icon};
use mcore::fuzzymatch::fuzzymatch;
use actions;


pub struct Context {
    /// Reference data for quick-send
    pub reference: Option<ItemData>,
    /// Candidates items list
    pub list_items: Vec<Item>,

    /// Stack of history items, init with empty stack
    /// Calling the last item's action would yields list_items
    history_items: Vec<Item>,

    /// Cached all actions
    all_actions: Vec<Arc<Box<Action + Sync + Send>>>,
}


impl Context {

    /// Create context with initial items
    pub fn new(config: toml::Value) -> Context {
        let mut ctx = Context {
            reference: None,
            list_items: Vec::new(),
            history_items: Vec::new(),
            all_actions: actions::get_actions(config),
        };
        ctx.reset();
        ctx
    }

    /// Reset context to initial state
    pub fn reset(&mut self) {
        self.reference = None;
        self.list_items = self.all_actions.iter()
            .filter(|action| {
                action.accept_nothing() || action.accept_text()
            })
            .map(|action| Item::new_action_item(action.clone()))
            .collect();
        self.list_items.sort_by_key(|item| item.priority );
        self.history_items = Vec::new();
    }

    /// Initialize quicksend item from clipboard
    #[cfg(not(feature="use-gtk"))]
    pub fn quicksend_from_clipboard(&mut self) -> Result<(), Box<Error + Sync + Send>> {
        let clip = Command::new("xclip").arg("-o").output()?;
        let clip = String::from_utf8(clip.stdout)?;
        if clip.len() > 0 {
            let clip_item = Item::new_text_item(&clip);
            self.quicksend(clip_item)
        } else {
            Ok(())
        }
    }

    #[cfg(feature="use-gtk")]
    pub fn quicksend_from_clipboard(&mut self) -> Result<(), Box<Error + Sync + Send>> {
        for clipboard in vec!["PRIMARY", "CLIPBOARD"] {
            let clipboard = gtk::Clipboard::get(&gdk::Atom::intern(&clipboard));
            let content = clipboard.wait_for_text();

            if let Some(text) = content {
                trace!("Clipboard content from: {:?}", text);
                return self.quicksend(Item::new_text_item(&text));
            }
        }
        Ok(())
    }

    #[cfg(not(feature="use-gtk"))]
    pub fn copy_content_to_clipboard(&self, item: &Item) -> Result<(), Box<Error + Sync + Send>> {
        let s : &str = match item.data {
            Some(ItemData::Text(ref text)) => text,
            Some(ItemData::Path(ref path)) => &path.to_str().unwrap(),
            _ => &item.title,
        };
        let mut cmd = Command::new("xclip");
        cmd.stdin(Stdio::piped());
        cmd.arg("-selection").arg("clipboard");
        let mut child = cmd.spawn()?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write(s.as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }

    #[cfg(feature="use-gtk")]
    pub fn copy_content_to_clipboard(&self, item: &Item) -> Result<(), Box<Error + Sync + Send>> {
        let s : &str = match item.data {
            Some(ItemData::Text(ref text)) => text,
            Some(ItemData::Path(ref path)) => &path.to_str().unwrap(),
            _ => &item.title,
        };
        let clipboard = gtk::Clipboard::get(&gdk::Atom::intern("CLIPBOARD"));
        clipboard.set_text(s);
        Ok(())
    }

    /// Filter list_items using fuzzymatch
    /// return indices of list_items
    pub fn filter(&self, pattern: &str) -> Vec<usize> {
        trace!("filter: {:?}", pattern);
        let scores = self.list_items.iter().map(|item| {
            let search_str = if let Some(ref search_str) = item.search_str {
                search_str
            } else {
                &item.title
            };
            fuzzymatch(search_str, pattern, false)
        });
        let mut indices_and_scores = (0..self.list_items.len()).zip(scores.into_iter())
            .collect::<Vec<(usize, i32)>>();
        indices_and_scores.sort_by_key(|index_and_score| -index_and_score.1);
        indices_and_scores.into_iter()
            .filter(|index_and_score| index_and_score.1 > 0)
            .map(|index_and_score| index_and_score.0)
            .collect::<Vec<usize>>()
    }

    pub fn selectable(&self, item: &Item) -> bool {
        if let Some(ref action) = item.action {
            action.accept_arg(&item.action_arg)
        } else {
            false
        }
    }

    pub fn selectable_with_text(&self, item: &Item) -> bool {
        if let Some(ref action) = item.action {
            item.action_arg.is_none() && action.accept_text()
        } else {
            false
        }
    }

    pub fn async_select_callback(&mut self, items: Vec<Item>) {
        self.list_items = items;
        self.list_items.sort_by_key(|x| x.priority);
        self.reference = None;
    }

    pub fn async_select<F>(&self, item: Item, callback: F) -> String
    where F: FnOnce(ActionResult) + Send + 'static {
        if !self.selectable(&item) {
            panic!("Item {} is not selectable", item);
        }
        let thread_uuid = Uuid::new_v4().simple().to_string();
        thread::Builder::new()
            .name(thread_uuid.clone())
            .spawn(move || {
                let action = item.action.unwrap();
                let items = action.run_arg(&item.action_arg);
                debug!("async select complete, calling back");
                callback(items);
            })
            .unwrap();
        thread_uuid
    }

    pub fn async_select_with_text<F>(&self, item: Item, text: &str, callback: F) -> String
    where F: FnOnce(ActionResult) + Send + 'static {
        if !self.selectable_with_text(&item) {
            panic!("Item {} is not selectable with text", &item);
        }
        let text = text.to_string();
        let thread_uuid = Uuid::new_v4().simple().to_string();
        thread::Builder::new()
            .name(thread_uuid.clone())
            .spawn(move || {
                let action = item.action.unwrap();
                let items = action.run_text(&text);
                debug!("async select with text complete, calling back");
                callback(items);
            })
            .unwrap();
        thread_uuid
    }

    pub fn select(&mut self, item: Item) -> Result<(), Box<Error + Send + Sync>> {
        if !self.selectable(&item) {
            panic!("Item {} is not selectable", item);
        }
        if let Some(ref action) = item.action {
            self.list_items = action.run_arg(&item.action_arg)?;
            self.list_items.sort_by_key(|item| item.priority );
        } else {
            panic!("Should not reach here");
        }
        self.history_items.push(item);
        self.reference = None;
        Ok(())
    }

    pub fn select_with_text(&mut self, item: Item, text: &str) -> Result<(), Box<Error + Send + Sync>> {
        if !self.selectable_with_text(&item) {
            panic!("Item {} is not selectable with text", &item);
        }
        if let Some(ref action) = item.action {
            self.list_items = action.run_text(text)?;
            self.list_items.sort_by_key(|item| item.priority );
        } else {
            panic!("Should not reach here");
        }
        self.history_items.push(item);
        self.reference = None;
        Ok(())
    }

    pub fn quicksend_able(&self, item: &Item) -> bool {
        self.reference.is_none() && item.data.is_some()
    }

    pub fn quicksend(&mut self, item: Item) -> Result<(), Box<Error + Send + Sync>> {
        if !self.quicksend_able(&item) {
            panic!("Item {} is not quicksend_able", item);
        }
        if let Some(ref data) = item.data {
            self.list_items = match data {
                &ItemData::Text(ref text) => {
                    self.all_actions.iter()
                    .filter(|action| action.accept_text())
                    .map(|action| {
                        let mut item = Item::new_action_item(action.clone());
                        item.action_arg = ActionArg::Text(text.clone());
                        item
                    })
                    .collect()
                },
                &ItemData::Path(ref path) => {
                    self.all_actions.iter()
                    .filter(|action| action.accept_path())
                    .map(|action| {
                        let mut item = Item::new_action_item(action.clone());
                        item.action_arg = ActionArg::Path(path.clone());
                        item
                    })
                    .collect()
                },
            };
            self.list_items.sort_by_key(|item| item.priority );
            self.reference = Some(data.clone());
        } else {
            panic!("Should not reach here");
        }
        Ok(())
    }

    pub fn back(&mut self) -> Result<(), Box<Error + Send + Sync>> {
        if let Some(action_item) = self.history_items.pop() {
            self.select(action_item)
        } else {
            self.reset();
            Ok(())
        }
    }
}

