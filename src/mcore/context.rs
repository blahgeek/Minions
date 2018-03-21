/*
* @Author: BlahGeek
* @Date:   2017-04-20
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-03-22
*/

extern crate gtk;
extern crate gdk;

extern crate uuid;
use self::uuid::Uuid;

use std::sync::Arc;
use std::thread;
use std::error::Error;
use std::rc::Rc;
use mcore::action::{ActionResult, PartialAction};
use mcore::item::Item;
use mcore::config::Config;
use mcore::lrudb::LruDB;
use actions;

use self::gtk::ClipboardExt;


pub struct Context {
    /// Reference data for quick-send
    pub reference: Option<String>,
    /// Candidates items list
    pub list_items: Vec<Rc<Item>>,

    /// Cached all actions
    action_items: Vec<Rc<Item>>,

    lrudb: Arc<LruDB>,
    history_max_n: i32,
}


impl Context {

    /// Create context with initial items
    pub fn new(config: &Config) -> Context {
        let db_file = config.get_filename(&["core", "db_file"]).unwrap();
        let history_max_n = config.get::<i32>(&["core", "history"]).unwrap();

        let mut ctx = Context {
            reference: None,
            list_items: Vec::new(),
            action_items: Vec::new(),
            lrudb: Arc::new(LruDB::new(Some(&db_file)).unwrap()),
            history_max_n: history_max_n,
        };
        ctx.reload(config);
        ctx.reset();
        ctx
    }

    /// Reload all action items
    pub fn reload(&mut self, config: &Config) {
        self.action_items = actions::get_action_items(config).into_iter()
            .map(|x| Rc::new(x)).collect();
    }

    /// Reset context to initial state
    pub fn reset(&mut self) {
        self.reference = None;
        self.list_items = self.action_items.clone();
        self.list_items.sort_by_key(|item| item.priority );
    }

    pub fn quicksend_from_clipboard(&mut self) -> Result<(), Box<Error + Sync + Send>> {
        for clipboard in vec!["PRIMARY", "CLIPBOARD"] {
            let clipboard = gtk::Clipboard::get(&gdk::Atom::intern(&clipboard));
            let content = clipboard.wait_for_text();

            if let Some(text) = content {
                trace!("Clipboard content from: {:?}", text);
                return self.quicksend(&Item::new_text_item(&text));
            }
        }
        Ok(())
    }

    pub fn copy_content_to_clipboard(&self, item: &Item) -> Result<(), Box<Error + Sync + Send>> {
        let clipboard = gtk::Clipboard::get(&gdk::Atom::intern("CLIPBOARD"));
        clipboard.set_text(item.data.as_ref().unwrap_or(&item.title));
        Ok(())
    }

    pub fn selectable(&self, item: &Item) -> bool {
        if let Some(ref action) = item.action {
            (action.runnable_arg() && self.reference.is_some())
                || action.runnable_bare()
        } else { false }
    }

    pub fn selectable_with_text(&self, item: &Item) -> bool {
        if let Some(ref action) = item.action {
            action.runnable_arg() && self.reference.is_none()
        } else { false }
    }

    pub fn runnable_with_text_realtime(&self, item: &Item) -> bool {
        if let Some(ref action) = item.action {
            action.runnable_arg_realtime()
        } else { false }
    }

    pub fn async_select_callback(&mut self, items: Vec<Item>) {
        self.list_items = items.into_iter().map(|x| Rc::new(x)).collect();
        self.list_items.sort_by_key(|x| x.priority);
        self.reference = None;
    }

    pub fn async_select<F>(&self, item: &Item, callback: F) -> String
    where F: FnOnce(ActionResult) + Send + 'static {
        assert!(self.selectable(item));
        let thread_uuid = Uuid::new_v4().simple().to_string();
        let action = item.action.clone().unwrap();
        let action_arg = self.reference.clone();
        thread::Builder::new()
            .name(thread_uuid.clone())
            .spawn(move || {
                let items =
                    if let Some(arg) = action_arg {
                        action.run_arg(&arg)
                    } else {
                        action.run_bare()
                    };
                debug!("async select complete, calling back");
                callback(items);
            })
            .unwrap();
        thread_uuid
    }

    pub fn async_select_with_text<F>(&self, item: &Item, text: &str, callback: F) -> String
    where F: FnOnce(ActionResult) + Send + 'static {
        assert!(self.selectable_with_text(&item));

        let text = text.to_string();
        let thread_uuid = Uuid::new_v4().simple().to_string();
        let action = item.action.clone().unwrap();

        if let Some(scope) = action.suggest_arg_scope() {
            if let Err(error) = self.lrudb.add(scope, &text, self.history_max_n) {
                warn!("Unable to save arg history: {}", error);
            }
        }

        thread::Builder::new()
            .name(thread_uuid.clone())
            .spawn(move || {
                let items = action.run_arg(&text);
                debug!("async select with text complete, calling back");
                callback(items);
            })
            .unwrap();
        thread_uuid
    }

    pub fn async_run_with_text_realtime<F>(&self, item: &Item, text: &str, callback: F) -> String
    where F: FnOnce(ActionResult) + Send + 'static {
        assert!(self.runnable_with_text_realtime(&item));
        let text = text.to_string();
        let thread_uuid = Uuid::new_v4().simple().to_string();
        let action = item.action.clone().unwrap();
        thread::Builder::new()
            .name(thread_uuid.clone())
            .spawn(move || {
                let items = action.run_arg_realtime(&text);
                debug!("async run with realtime text complete, calling back");
                callback(items);
            })
            .unwrap();
        thread_uuid
    }

    pub fn quicksend(&mut self, item: &Item) -> Result<(), Box<Error + Send + Sync>> {
        self.list_items =
            self.action_items.iter()
            .filter(|item| {
                item.action.as_ref().unwrap().runnable_arg()
            }).map(|x| x.clone()).collect();
        self.list_items.sort_by_key(|item| item.priority );
        self.reference = Some(item.data.as_ref().unwrap_or(&item.title).clone());
        Ok(())
    }

    pub fn suggest_arg(&self, item: &Item) -> Vec<Item> {
        if let Some(ref action) = item.action {
            if let Some(scope) = action.suggest_arg_scope() {
                if let Ok(history) = self.lrudb.getall(scope) {
                    history.into_iter().map(|x| {
                        let mut sug = Item::new(&x.data);
                        sug.subtitle = Some(x.time.format("%T %b %e").to_string());
                        sug.icon = item.icon.clone();
                        sug.badge = Some("History".into());

                        let lrudb = self.lrudb.clone();
                        let scope : String = scope.into();
                        let arg = x.data.clone();
                        let history_max_n = self.history_max_n;
                        sug.action = Some(Arc::new(Box::new(PartialAction::new(
                                        action.clone(), x.data,
                                        Some(Box::new(move || {
                                            if let Err(error) = lrudb.add(&scope, &arg, history_max_n) {
                                                warn!("Unable to save arg history: {}", error);
                                            }
                                        }))
                                        ))));
                        sug
                    }).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

}

