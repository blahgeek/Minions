/*
* @Author: BlahGeek
* @Date:   2017-04-23
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-26
*/

extern crate uuid;
use self::uuid::Uuid;

use toml;

use std;
use frontend_gtk::gdk;
use frontend_gtk::gtk;
use frontend_gtk::gtk::prelude::*;

use std::thread;
use std::rc::Rc;
use std::sync::Arc;
use std::ops::Deref;
use std::cell::RefCell;

use frontend_gtk::ui::MinionsUI;
use mcore::context::Context;
use mcore::item::Item;

#[derive(Clone)]
enum Status {
    Initial,
    // Running(String),  // running in another thread with thread name
    FilteringNone,
    FilteringEntering {
        selected_idx: i32,
        filter_text: String,
        filter_text_lasttime: std::time::Instant,
        filter_indices: Vec<usize>,
    },
    FilteringMoving {
        selected_idx: i32,
        filter_text: String,
        filter_indices: Vec<usize>,
    },
    EnteringText(usize), // entering text for item, (index for list_items)
}

pub struct MinionsApp {
    ui: MinionsUI,
    ctx: Context,

    status: Status,
    // app: Option<Arc<RefCell<MinionsApp>>>,  // itself
}

impl MinionsApp {

    fn update_ui(&self, refresh_items: bool) {
        trace!("update ui");
        match self.status {
            Status::Initial => {
                self.ui.set_entry(None);
                self.ui.set_filter_text("");
                self.ui.set_reference_item(None);
                self.ui.set_items(Vec::new(), &self.ctx);
                self.ui.set_highlight_item(-1);
            },
            Status::FilteringNone => {
                self.ui.set_entry(None);
                self.ui.set_filter_text("");
                self.ui.set_reference_item(match self.ctx.reference_item {
                    None => None,
                    Some(ref item) => Some(&item),
                });
                if self.ctx.list_items.len() == 0 {
                    warn!("No more listing items!");
                    gtk::main_quit();
                }
                if refresh_items {
                    // self.ui.set_items(self.ctx.list_items.iter().map(|x| x.deref()).collect::<Vec<&Item>>(), &self.ctx);
                    self.ui.set_items(self.ctx.list_items.iter().collect(), &self.ctx);
                }
                self.ui.set_highlight_item(-1);
            },
            Status::FilteringEntering {
                selected_idx,
                ref filter_text,
                filter_text_lasttime: _,
                ref filter_indices
            } |
            Status::FilteringMoving {
                selected_idx,
                ref filter_text,
                ref filter_indices
            } => {
                if selected_idx < 0 {
                    self.ui.set_entry(None);
                } else {
                    self.ui.set_entry(Some(&self.ctx.list_items[filter_indices[selected_idx as usize]]))
                }
                self.ui.set_filter_text(&filter_text);
                self.ui.set_reference_item(match self.ctx.reference_item {
                    None => None,
                    Some(ref item) => Some(&item),
                });
                if refresh_items {
                    self.ui.set_items(filter_indices.iter().map(|x| &self.ctx.list_items[x.clone()])
                                      .collect::<Vec<&Item>>(), &self.ctx);
                }
                self.ui.set_highlight_item(selected_idx);
            },
            Status::EnteringText(idx) => {
                self.ui.set_entry(None);
                self.ui.set_entry_editable();
                self.ui.set_filter_text("");
                self.ui.set_reference_item(Some(&self.ctx.list_items[idx]));
                self.ui.set_items(Vec::new(), &self.ctx);
                self.ui.set_highlight_item(-1);
            }
        }
    }

    fn process_timeout(&mut self) {
        if let Status::FilteringEntering {
            selected_idx: _,
            filter_text: _,
            filter_text_lasttime,
            filter_indices: _
        } = self.status {
            if filter_text_lasttime.elapsed() > std::time::Duration::new(1, 0) {
                self.status = Status::FilteringNone;
                self.update_ui(true);
            }
        }
    }

    fn process_keyevent_escape(&mut self) {
        debug!("Processing keyevent Escape");
        self.status = match self.status {
            Status::Initial => {
                debug!("Quit!");
                gtk::main_quit();
                Status::Initial
            },
            Status::FilteringNone => {
                self.ctx.reset();
                Status::Initial
            },
            _ => Status::FilteringNone,
        };
        self.update_ui(true);
    }

    fn process_keyevent_move(&mut self, delta: i32) {
        debug!("Processing keyevent Move: {}", delta);
        self.status = match self.status.clone() {
            Status::FilteringNone => {
                Status::FilteringMoving {
                    selected_idx: 0,
                    filter_text: String::new(),
                    filter_indices: (0..self.ctx.list_items.len()).collect(),
                }
            },
            Status::FilteringEntering {
                selected_idx,
                filter_text,
                filter_text_lasttime: _,
                filter_indices
            } |
            Status::FilteringMoving {
                selected_idx,
                filter_text,
                filter_indices
            } => {
                let mut new_idx = selected_idx + delta;
                if filter_indices.len() == 0 {
                    new_idx = -1;
                } else {
                    if new_idx >= filter_indices.len() as i32 {
                        new_idx = filter_indices.len() as i32 - 1;
                    }
                    if new_idx < 0 {
                        new_idx = 0;
                    }
                }
                Status::FilteringMoving {
                    selected_idx: new_idx,
                    filter_text: filter_text,
                    filter_indices: filter_indices
                }
            },
            status @ _ => status,
        };
        self.update_ui(false);
    }

    fn process_keyevent_tab(&mut self) {
        debug!("Processing keyevent Tab");
        self.status = match self.status.clone() {
            Status::FilteringEntering {
                selected_idx,
                filter_text: _,
                filter_text_lasttime: _,
                filter_indices
            } |
            Status::FilteringMoving {
                selected_idx,
                filter_text: _,
                filter_indices
            } => {
                if selected_idx < 0 {
                    warn!("No item to send");
                    self.status.clone()
                } else {
                    let item = self.ctx.list_items[filter_indices[selected_idx as usize]].clone();
                    if self.ctx.quicksend_able(&item) {
                        if let Err(error) = self.ctx.quicksend(item) {
                            warn!("Unable to quicksend item: {}", error);
                            self.status.clone()
                        } else {
                            Status::FilteringNone
                        }
                    } else {
                        warn!("Item not sendable");
                        self.status.clone()
                    }
                }
            },
            status @ _ => status,
        };
        self.update_ui(true);
    }

    fn _make_status_filteringentering(&self, text: String) -> Status {
        let filter_indices = self.ctx.filter(&text);
        let selected_idx = if filter_indices.len() == 0 { -1 } else { 0 };
        Status::FilteringEntering {
            selected_idx: selected_idx,
            filter_text: text,
            filter_text_lasttime: std::time::Instant::now(),
            filter_indices: filter_indices,
        }
    }

    fn process_keyevent_char(&mut self, ch: char) {
        debug!("Processing keyevent Char: {}", ch);
        let mut should_update_ui = true;
        self.status = match self.status.clone() {
            Status::Initial | Status::FilteringNone => {
                let mut text = String::new();
                text.push(ch);
                self._make_status_filteringentering(text)
            },
            Status::FilteringEntering {
                selected_idx: _,
                mut filter_text,
                filter_text_lasttime: _,
                filter_indices: _
            } |
            Status::FilteringMoving {
                selected_idx: _,
                mut filter_text,
                filter_indices: _,
            } => {
                filter_text.push(ch);
                self._make_status_filteringentering(filter_text)
            },
            status @ _ => {
                should_update_ui = false;
                status
            },
        };
        if should_update_ui {
            self.update_ui(true);
        }
    }

    fn process_keyevent_space(&mut self) {
        debug!("Processing keyevent Space");
        let mut should_update_ui = false;
        self.status = match self.status.clone() {
            Status::FilteringEntering {
                selected_idx,
                filter_text: _,
                filter_text_lasttime: _,
                filter_indices
            } |
            Status::FilteringMoving {
                selected_idx,
                filter_text: _,
                filter_indices
            } => {
                if selected_idx < 0 {
                    warn!("No item to select");
                    self.status.clone()
                } else {
                    let idx = filter_indices[selected_idx as usize];
                    let item = &self.ctx.list_items[idx];
                    if self.ctx.selectable_with_text(item) {
                        should_update_ui = true;
                        Status::EnteringText(idx)
                    } else {
                        warn!("Item not selectable with or without text");
                        self.status.clone()
                    }
                }
            },
            status @ _ => status,
        };

        if should_update_ui {
            self.update_ui(true);
        }
    }

    fn process_keyevent_enter(&mut self) {
        debug!("Processing keyevent Enter");
        self.status = match self.status.clone() {
            status @ Status::Initial | status @ Status::FilteringNone => status,
            Status::FilteringEntering {
                selected_idx,
                filter_text: _,
                filter_text_lasttime: _,
                filter_indices
            } |
            Status::FilteringMoving {
                selected_idx,
                filter_text: _,
                filter_indices
            } => {
                if selected_idx < 0 {
                    warn!("No item to select");
                    self.status.clone()
                } else {
                    let idx = filter_indices[selected_idx as usize];
                    let item = self.ctx.list_items[idx].clone();
                    // let thread_uuid = Uuid::new_v4().simple().to_string();
                    if self.ctx.selectable(&item) {
                        if let Err(error) = self.ctx.select(item) {
                            warn!("Unable to select item: {}", error);
                            self.status.clone()
                        } else {
                            Status::FilteringNone
                        }
                    } else if self.ctx.selectable_with_text(&item) {
                        Status::EnteringText(idx)
                    } else {
                        warn!("Item not selectable with or without text");
                        self.status.clone()
                    }
                }
            },
            Status::EnteringText(idx) => {
                let text = self.ui.get_entry_text();
                let item = self.ctx.list_items[idx].clone();
                if let Err(error) = self.ctx.select_with_text(item, &text) {
                    warn!("Unable to select item with text: {}", error);
                    Status::EnteringText(idx)
                } else {
                    Status::FilteringNone
                }
            },
        };
        self.update_ui(true);
    }

    fn process_keyevent(&mut self, event: &gdk::EventKey) -> Inhibit {
        let key = event.get_keyval();
        let modi = event.get_state();
        debug!("Key pressed: {:?}/{:?}", key, modi);
        if key == gdk::enums::key::Return {
            self.process_keyevent_enter();
            Inhibit(true)
        } else if key == gdk::enums::key::space {
            self.process_keyevent_space();
            Inhibit(false)
        } else if key == gdk::enums::key::Escape {
            self.process_keyevent_escape();
            Inhibit(true)
        } else if key == gdk::enums::key::Tab {
            self.process_keyevent_tab();
            Inhibit(true)
        } else if key == 'j' as u32 && modi == gdk::CONTROL_MASK {
            self.process_keyevent_move(1);
            Inhibit(true)
        } else if key == 'k' as u32 && modi == gdk::CONTROL_MASK {
            self.process_keyevent_move(-1);
            Inhibit(true)
        } else if key == gdk::enums::key::Down {
            self.process_keyevent_move(1);
            Inhibit(true)
        } else if key == gdk::enums::key::Up {
            self.process_keyevent_move(-1);
            Inhibit(true)
        } else if let Some(ch) = gdk::keyval_to_unicode(key) {
            if ch.is_alphabetic() {
                self.process_keyevent_char(ch);
            } else {
                debug!("Ignore char: {}", ch);
            }
            Inhibit(false)
        } else {
            Inhibit(false)
        }
    }

    pub fn new(config: toml::Value, from_clipboard: bool) -> Arc<RefCell<MinionsApp>> {
        let mut app = MinionsApp {
            ui: MinionsUI::new(),
            ctx: Context::new(config),
            status: Status::Initial,
            // app: None,
        };
        if from_clipboard {
            if let Err(error) = app.ctx.quicksend_from_clipboard() {
                warn!("Unable to get content from clipboard: {}", error);
            } else {
                app.status = Status::FilteringNone;
            }
        }
        app.update_ui(true);

        let app = Arc::new(RefCell::new(app));
        // app.borrow_mut().app = Some(app.clone());

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
