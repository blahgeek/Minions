/*
* @Author: BlahGeek
* @Date:   2017-04-23
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-05-01
*/

use std;
use gdk;
use gtk;
use gtk::prelude::*;

use std::rc::Rc;
use std::cell::RefCell;

use frontend::ui::MinionsUI;
use mcore::context::Context;

pub struct MinionsApp {
    ui: MinionsUI,
    ctx: Context,

    // filter related
    filter_text: String,
    filter_text_lasttime: std::time::Instant,
}

impl MinionsApp {

    fn process_keyevent(&mut self, event: &gdk::EventKey) -> Inhibit {
        let key = event.get_keyval();
        println!("Key pressed: {:?}", key);
        if let Some(ch) = gdk::keyval_to_unicode(key) {
            if ch.is_alphabetic() {
                self.filter_text.push(ch);
                self.filter_text_lasttime = std::time::Instant::now();
                self.ui.set_filter_text(&self.filter_text);
                self.ui.set_items(&self.ctx.filter(&self.filter_text));
            }
        }
        Inhibit(false)
    }

    fn process_timeout(&mut self) {
        let duration = self.filter_text_lasttime.elapsed();
        if duration > std::time::Duration::new(1, 0) && self.filter_text.len() > 0 {
            self.filter_text = String::new();
            self.ui.set_filter_text("");
            self.ui.set_items(&self.ctx.list_items.iter().collect());
        }
    }

    pub fn new() -> Rc<RefCell<MinionsApp>> {
        let app = MinionsApp {
            ui: MinionsUI::new(),
            ctx: Context::new(),
            filter_text: String::new(),
            filter_text_lasttime: std::time::Instant::now(),
        };
        app.ui.set_items(&app.ctx.list_items.iter().collect());
        app.ui.set_reference_item(&app.ctx.reference_item);

        let app = Rc::new(RefCell::new(app));
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
