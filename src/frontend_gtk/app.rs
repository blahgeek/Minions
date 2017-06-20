/*
* @Author: BlahGeek
* @Date:   2017-04-23
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-20
*/

use toml;

use std;
use frontend_gtk::gdk;
use frontend_gtk::gtk;
use frontend_gtk::gtk::prelude::*;

use std::rc::Rc;
use std::ops::Deref;
use std::cell::RefCell;

use frontend_gtk::ui::MinionsUI;
use mcore::context::Context;
use mcore::item::Item;

pub struct MinionsApp {
    ui: MinionsUI,
    ctx: Context,

    // filter related
    filter_text: String,
    filter_text_lasttime: std::time::Instant,
    filter_text_should_reset: bool,

    filtered_items: Vec<Rc<Item>>,

    highlighting_idx: isize,
}

impl MinionsApp {

    fn reset_to_ctx(&mut self) {
        self.filter_text = String::new();
        self.update_filter();
        self.ui.set_reference_item(match self.ctx.reference_item {
            None => None,
            Some(ref item) => Some(&item),
        });
    }

    fn update_filter(&mut self) {
        self.filter_text_lasttime = std::time::Instant::now();
        self.filter_text_should_reset = true;
        self.ui.set_filter_text(&self.filter_text);
        if self.filter_text.len() == 0 {
            self.filtered_items = self.ctx.list_items.clone();
            self.highlighting_idx = -1;
        } else {
            self.filtered_items = self.ctx.filter(&self.filter_text);
            self.highlighting_idx = 0;
        }
        self.ui.set_items(self.filtered_items.iter().map(|x| x.deref()).collect::<Vec<&Item>>());
        self.update_highlight();
    }

    fn update_highlight(&mut self) {
        if self.highlighting_idx < 0 || self.highlighting_idx >= self.filtered_items.len() as isize {
            self.highlighting_idx = -1;
            self.ui.set_entry(None);
            self.ui.set_highlight_item(-1);
            return;
        }

        self.ui.set_highlight_item(self.highlighting_idx as i32);
        self.ui.set_entry(Some(&self.filtered_items[self.highlighting_idx as usize]));
    }

    fn enter_item(&mut self) {
        debug!("Enter item!");
        if self.highlighting_idx < 0 {
            return;
        }
        let item = self.filtered_items[self.highlighting_idx as usize].clone();
        if !self.ctx.selectable(&item) {
            return;
        }
        self.ctx.select(item).expect("should run success");
    }

    fn process_keyevent(&mut self, event: &gdk::EventKey) -> Inhibit {
        let key = event.get_keyval();
        debug!("Key pressed: {:?}", key);
        if key == gdk::enums::key::Return {
            self.enter_item();
        } else if key == gdk::enums::key::Escape {
            self.reset_to_ctx();
        } else if key == gdk::enums::key::Down {
            self.highlighting_idx += 1;
            self.filter_text_should_reset = false;
            self.update_highlight();
        } else if key == gdk::enums::key::Up {
            self.highlighting_idx -= 1;
            self.filter_text_should_reset = false;
            self.update_highlight();
        } else if let Some(ch) = gdk::keyval_to_unicode(key) {
            if ch.is_alphabetic() {
                self.filter_text.push(ch);
                self.update_filter();
            }
        }
        Inhibit(false)
    }

    fn process_timeout(&mut self) {
        let duration = self.filter_text_lasttime.elapsed();
        if self.filter_text_should_reset &&
           duration > std::time::Duration::new(1, 0) &&
           self.filter_text.len() > 0 {
            self.filter_text = String::new();
            self.update_filter();
        }
    }

    pub fn new(config: toml::Value, from_clipboard: bool) -> Rc<RefCell<MinionsApp>> {
        let mut app = MinionsApp {
            ui: MinionsUI::new(),
            ctx: Context::new(config),
            filter_text: String::new(),
            filter_text_lasttime: std::time::Instant::now(),
            filter_text_should_reset: true,
            filtered_items: Vec::new(),
            highlighting_idx: 0,
        };
        if from_clipboard {
            if let Err(error) = app.ctx.quicksend_from_clipboard() {
                warn!("Unable to get content from clipboard: {}", error);
            }
        }
        app.reset_to_ctx();

        let app  = Rc::new(RefCell::new(app));
        let app_ = app.clone();
        app.borrow().ui.window.connect_key_press_event(move |_, event| {
            app_.borrow_mut().process_keyevent(event)
        });

        let app_ = app.clone();
        gtk::timeout_add(200, move || {
            app_.borrow_mut().process_timeout();
            Continue(true)
        });

        app
    }
}
