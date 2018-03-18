/*
* @Author: BlahGeek
* @Date:   2017-04-23
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-03-18
*/

extern crate glib;
extern crate libc;
extern crate glib_sys;

use std;
use std::ffi;
use std::error::Error;
use frontend::gdk;
use frontend::gtk;
use frontend::gtk::prelude::*;

use std::thread;
use std::sync::mpsc;
use std::cell::RefCell;
use std::rc::Rc;
use std::ops::Deref;

use frontend::ui::MinionsUI;
use mcore::context::Context;
use mcore::action::ActionResult;
use mcore::item::Item;
use mcore::matcher::Matcher;
use mcore::config::Config;

#[derive(Clone)]
enum Status {
    Initial,
    Running(Rc<mpsc::Receiver<ActionResult>>),
    Error(Rc<Box<Error>>), // Rc is for Clone
    Default,
    Filtering {
        selected_idx: i32,
        filter_text: String,
        filtered_items: Vec<Rc<Item>>,
        timestamp: std::time::Instant,
    },
    Entering {
        item: Rc<Item>, // entering text for item
        suggestions: Vec<Rc<Item>>,
        selected_idx: i32, // selected index of suggestions
        receiver: Option<Rc<mpsc::Receiver<ActionResult>>>, // receiver for running suggestion
    },
}

pub struct MinionsApp {
    ui: MinionsUI,
    ctx: Context,

    status: Status,
    filter_timeout: std::time::Duration,
    matcher: Matcher,
}


thread_local! {
    static APP: RefCell<Option<MinionsApp>> = RefCell::new(None);
}


#[link(name="keybinder-3.0")]
extern {
    fn keybinder_init();
    fn keybinder_bind(keystring: *const libc::c_char,
                      handler: extern fn(*const libc::c_char, *mut libc::c_void),
                      user_data: *mut libc::c_void) -> glib_sys::gboolean;
}

extern fn keybinder_callback_show(_: *const libc::c_char, _: *mut libc::c_void) {
    trace!("keybinder callback: show");
    glib::idle_add( move || {
        APP.with(|app| {
            if let Some(ref mut app) = *app.borrow_mut() {
                app.reset_window(false);
            }
        });
        Continue(false)
    });
}

extern fn keybinder_callback_show_clipboard(_: *const libc::c_char, _: *mut libc::c_void) {
    trace!("keybinder callback: show with clipboard");
    glib::idle_add( move || {
        APP.with(|app| {
            if let Some(ref mut app) = *app.borrow_mut() {
                app.reset_window(true);
            }
        });
        Continue(false)
    });
}

impl MinionsApp {

    fn update_ui(&self) {
        trace!("update ui");
        match self.status {
            Status::Initial => {
                self.ui.set_entry(None);
                self.ui.set_filter_text("");
                self.ui.set_action(None);
                self.ui.set_reference(None);
                self.ui.set_items(Vec::new(), -1, &self.ctx);
                self.ui.set_spinning(false);
            },
            Status::Running(_) => {
                self.ui.set_entry(None);
                self.ui.set_filter_text("");
                self.ui.set_action(None);
                self.ui.set_reference(None);
                self.ui.set_items(Vec::new(), -1, &self.ctx);
                self.ui.set_spinning(true);
            },
            Status::Error(ref error) => {
                self.ui.set_entry(None);
                self.ui.set_filter_text("");
                self.ui.set_action(None);
                self.ui.set_reference(None);
                self.ui.set_items(Vec::new(), -1, &self.ctx);
                self.ui.set_spinning(false);
                self.ui.set_error(&error);
            },
            Status::Default => {
                self.ui.set_spinning(false);
                self.ui.set_entry(None);
                self.ui.set_filter_text("");
                self.ui.set_action(None);
                self.ui.set_reference(self.ctx.reference.as_ref());
                if self.ctx.list_items.len() == 0 {
                    debug!("No more listing items!");
                    self.ui.window.hide();
                }
                self.ui.set_items(self.ctx.list_items.iter().map(|x| x.deref()).collect(), -1, &self.ctx);
            },
            Status::Filtering {
                selected_idx,
                ref filter_text,
                ref filtered_items,
                ..
            } => {
                if selected_idx < 0 {
                    self.ui.set_entry(None);
                } else {
                    self.ui.set_entry(Some(&filtered_items[selected_idx as usize]))
                }
                self.ui.set_spinning(false);
                self.ui.set_filter_text(&filter_text);
                self.ui.set_action(None);
                self.ui.set_reference(self.ctx.reference.as_ref());
                self.ui.set_items(filtered_items.iter().map(|x| x.deref()).collect(),
                                  selected_idx, &self.ctx);
            },
            Status::Entering {
                ref item,
                ref suggestions,
                selected_idx,
                receiver: _
            } => {
                self.ui.set_spinning(false);
                // defer set_entry_editable to prevent a leading space to be inserted
                glib::timeout_add(50, move || {
                    APP.with(|app| {
                        if let Some(ref app) = *app.borrow() {
                            if let Status::Entering{..} = app.status {
                                app.ui.set_entry_editable();
                            }
                        }
                    });
                    Continue(false)
                });

                self.ui.set_filter_text("");
                self.ui.set_action(Some(&item));
                self.ui.set_reference(None);
                self.ui.set_items(suggestions.iter().map(|x| x.deref()).collect(), selected_idx, &self.ctx);
            },
        }
    }

    fn process_keyevent_escape(&mut self) {
        trace!("Processing keyevent Escape");
        self.status = match self.status {
            Status::Initial => {
                debug!("Quit!");
                self.ui.window.hide();
                Status::Initial
            },
            Status::Default => {
                self.ctx.reset();
                Status::Initial
            },
            Status::Running(_) => {
                debug!("Drop thread");
                Status::Default
            },
            _ => Status::Default,
        };
        self.update_ui();
    }

    fn process_keyevent_move(&mut self, delta: i32) {
        trace!("Processing keyevent Move: {}", delta);
        self.status = match self.status.clone() {
            Status::Default | Status::Initial => {
                Status::Filtering {
                    selected_idx: 0,
                    filter_text: String::new(),
                    filtered_items: self.ctx.list_items.clone(),
                    timestamp: std::time::Instant::now(),
                }
            },
            Status::Filtering {
                selected_idx,
                filter_text,
                filtered_items,
                timestamp,
            } => {
                let mut new_idx = selected_idx + delta;
                if filtered_items.len() == 0 {
                    new_idx = -1;
                } else {
                    if new_idx >= filtered_items.len() as i32 {
                        new_idx = filtered_items.len() as i32 - 1;
                    }
                    if new_idx < 0 {
                        new_idx = 0;
                    }
                }
                Status::Filtering {
                    selected_idx: new_idx,
                    filter_text: filter_text,
                    filtered_items: filtered_items,
                    timestamp: timestamp,
                }
            },
            Status::Entering {
                item,
                suggestions,
                selected_idx,
                ..
            } => {
                let mut new_idx = selected_idx + delta;
                if new_idx >= suggestions.len() as i32 {
                    new_idx = suggestions.len() as i32 - 1;
                }
                if new_idx < 0 {
                    new_idx = -1;
                }
                Status::Entering {
                    item: item,
                    suggestions: suggestions,
                    selected_idx: new_idx,
                    receiver: None, // drop receiver
                }
            },
            status @ _ => status,
        };
        self.update_ui();
    }

    fn process_keyevent_tab(&mut self) {
        trace!("Processing keyevent Tab");
        self.status = match self.status.clone() {
            Status::Filtering {
                selected_idx,
                ref filter_text,
                ref filtered_items,
                ..
            } => {
                if selected_idx < 0 || self.ctx.reference.is_some() {
                    debug!("No item to send or not sendable");
                    self.status.clone()
                } else {
                    let item = &filtered_items[selected_idx as usize];
                    if let Err(error) = self.matcher.record(Some(&filter_text), &item) {
                        warn!("Unable to record hit: {}", error);
                    }
                    if let Err(error) = self.ctx.quicksend(item) {
                        debug!("Unable to quicksend item: {}", error);
                        Status::Error(Rc::new(error))
                    } else {
                        Status::Default
                    }
                }
            },
            status @ _ => status,
        };
        self.update_ui();
    }

    fn process_keyevent_char(&mut self, ch: char) {
        trace!("Processing keyevent Char: {}", ch);

        let newfilter = match self.status {
            Status::Initial | Status::Default => {
                let mut text = String::new();
                text.push(ch);
                Some(text)
            },
            Status::Filtering {
                ref filter_text,
                ref timestamp,
                ..
            } => {
                let mut text = String::new();
                if (self.filter_timeout.as_secs() == 0 && self.filter_timeout.subsec_nanos() == 0)
                    || timestamp.elapsed() < self.filter_timeout {
                    text = filter_text.clone();
                }
                text.push(ch);
                Some(text)
            },
            _ => None,
        };

        if let Some(newfilter) = newfilter {
            let filtered_items = self.matcher.sort(&newfilter, &self.ctx.list_items);
            let selected_idx = if filtered_items.len() == 0 { -1 } else { 0 };

            self.status = Status::Filtering {
                selected_idx: selected_idx,
                filter_text: newfilter,
                filtered_items: filtered_items,
                timestamp: std::time::Instant::now(),
            };
            self.update_ui();
        }
    }

    fn process_keyevent_space(&mut self) {
        trace!("Processing keyevent Space");
        let mut should_update_ui = false;
        self.status = match self.status.clone() {
            Status::Filtering {
                selected_idx,
                ref filter_text,
                ref filtered_items,
                ..
            } => {
                if selected_idx < 0 {
                    debug!("No item to select");
                    self.status.clone()
                } else {
                    let item = &filtered_items[selected_idx as usize];
                    if let Err(error) = self.matcher.record(Some(&filter_text), &item) {
                        warn!("Unable to record hit: {}", error);
                    }
                    if self.ctx.selectable_with_text(item) {
                        should_update_ui = true;
                        Status::Entering{
                            item: item.clone(),
                            suggestions: self.ctx.suggest_arg(item)
                                .into_iter().map(|x| Rc::new(x)).collect(),
                            selected_idx: -1,
                            receiver: None,
                        }
                    } else {
                        debug!("Item not selectable with text");
                        self.status.clone()
                    }
                }
            },
            status @ _ => status,
        };

        if should_update_ui {
            self.update_ui();
        }
    }

    fn process_entry_text_changed(&mut self) {

        // only match if receiver is None
        if let Status::Entering{item, suggestions, receiver: None, ..} = self.status.clone() {
            let entry_text = self.ui.textentry.get_text().unwrap();
            trace!("Entry text changed: {}", &entry_text);

            if entry_text.len() > 0 && self.ctx.runnable_with_text_realtime(&item) {
                let (send_ch, recv_ch) = mpsc::channel::<ActionResult>();
                let entry_text_ = entry_text.clone();
                self.ctx.async_run_with_text_realtime(&item, &entry_text, move |res: ActionResult| {
                    if let Err(error) = send_ch.send(res) {
                        warn!("Unable to send to channel: {}", error);
                    } else {
                        glib::idle_add(move || {
                            APP.with(|app| app.borrow_mut().as_mut().unwrap().process_running_text_realtime_callback(&entry_text_));
                            Continue(false)
                        });
                    }
                });
                self.status = Status::Entering {
                    item: item,
                    suggestions: suggestions,
                    selected_idx: -1,
                    receiver: Some(Rc::new(recv_ch)),
                };
            }
        }
    }

    fn process_running_text_realtime_callback(&mut self, text: &str) {
        if let Status::Entering{item, suggestions, receiver: Some(receiver), ..} = self.status.clone() {
            if let Ok(res) = receiver.try_recv() {
                trace!("Received realtime text result on callback");
                self.status = match res {
                    Ok(res) => {
                        Status::Entering {
                            item: item,
                            suggestions: res.into_iter().map(|x| Rc::new(x)).collect(),
                            selected_idx: -1,
                            receiver: None
                        }
                    },
                    Err(error) => {
                        warn!("Error running realtime text: {}", error);
                        Status::Entering {
                            item: item,
                            suggestions: suggestions,
                            selected_idx: -1,
                            receiver: None,
                        }
                    }
                };
                self.update_ui();

                if text != self.ui.textentry.get_text().unwrap() {
                    self.process_entry_text_changed()
                }

            } else {
                debug!("Unable to receive realtime text result from channel");
            }
        } else {
            debug!("Invalid status on realtime text callback");
        }

    }

    fn process_running_callback(&mut self) {
        let mut res : Option<ActionResult> = None;
        if let Status::Running(ref recv_ch) = self.status {
            if let Ok(res_) = recv_ch.try_recv() {
                trace!("Received result on callback");
                res = Some(res_);
            } else {
                debug!("Unable to receive from channel");
            }
        }

        if let Some(res) = res {
            self.status = match res {
                Ok(res) => {
                    self.ctx.async_select_callback(res);
                    Status::Default
                },
                Err(error) => {
                    debug!("Error from channel: {}", error);
                    Status::Error(Rc::new(error))
                }
            };
            self.update_ui();
        } else {
            debug!("No action result");
        }
    }

    fn process_keyevent_enter(&mut self) {
        trace!("Processing keyevent Enter");
        self.status = match self.status.clone() {
            status @ Status::Initial | status @ Status::Default => status,
            Status::Filtering {
                selected_idx,
                ref filter_text,
                ref filtered_items,
                ..
            } => {
                if selected_idx < 0 {
                    debug!("No item to select");
                    self.status.clone()
                } else {
                    let item = &filtered_items[selected_idx as usize];
                    if let Err(error) = self.matcher.record(Some(&filter_text), &item) {
                        debug!("Unable to record hit: {}", error);
                    }

                    let (send_ch, recv_ch) = mpsc::channel::<ActionResult>();
                    if self.ctx.selectable(item) {
                        self.ctx.async_select(item, move |res: ActionResult| {
                            if let Err(error) = send_ch.send(res) {
                                debug!("Unable to send to channel: {}", error);
                            } else {
                                glib::idle_add( || {
                                    APP.with(move |app| app.borrow_mut().as_mut().unwrap().process_running_callback() );
                                    Continue(false)
                                });
                            }
                        });
                        Status::Running(Rc::new(recv_ch))
                    } else if self.ctx.selectable_with_text(&item) {
                        Status::Entering{
                            item: item.clone(),
                            suggestions: self.ctx.suggest_arg(item)
                                .into_iter().map(|x| Rc::new(x)).collect(),
                            selected_idx: -1,
                            receiver: None,
                        }
                    } else {
                        debug!("Item not selectable with or without text");
                        self.status.clone()
                    }
                }
            },
            Status::Entering{item, suggestions, selected_idx, ..} => {
                let (send_ch, recv_ch) = mpsc::channel::<ActionResult>();

                if selected_idx < 0 {
                    let text = self.ui.get_entry_text();
                    self.ctx.async_select_with_text(&item, &text, move |res: ActionResult| {
                        if let Err(error) = send_ch.send(res) {
                            debug!("Unable to send to channel: {}", error);
                        } else {
                            glib::idle_add( || {
                                APP.with(move |app| app.borrow_mut().as_mut().unwrap().process_running_callback() );
                                Continue(false)
                            });
                        }
                    });
                    Status::Running(Rc::new(recv_ch))
                } else {
                    let item = &suggestions[selected_idx as usize];
                    if self.ctx.selectable(&item) {
                        let (send_ch, recv_ch) = mpsc::channel::<ActionResult>();
                        self.ctx.async_select(&item, move |res: ActionResult| {
                            if let Err(error) = send_ch.send(res) {
                                debug!("Unable to send to channel: {}", error);
                            } else {
                                glib::idle_add( || {
                                    APP.with(move |app| app.borrow_mut().as_mut().unwrap().process_running_callback() );
                                    Continue(false)
                                });
                            }
                        });
                        Status::Running(Rc::new(recv_ch))
                    } else {
                        debug!("Item not selectable with nothing");
                        self.status.clone()
                    }
                }

            },
            status @ _ => status,
        };
        self.update_ui();
    }

    fn process_keyevent_copy(&mut self) {
        trace!("Process keyevent copy");
        self.status = match self.status.clone() {
            Status::Filtering {
                selected_idx,
                filtered_items,
                ..
            } => {
                if let Err(error) = self.ctx.copy_content_to_clipboard(&filtered_items[selected_idx as usize]) {
                    warn!("Unable to copy item: {}", error);
                } else {
                    info!("Item copied");
                }
                self.status.clone()
            },
            status @ _ => { status },
        };
        self.ui.window.hide();
    }

    fn process_keyevent(&mut self, event: &gdk::EventKey) -> Inhibit {
        let key = event.get_keyval();
        let modi = event.get_state();
        trace!("Key pressed: {:?}/{:?}", key, modi);
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
        } else if key == 'j' as u32 && modi == gdk::ModifierType::CONTROL_MASK {
            self.process_keyevent_move(1);
            Inhibit(true)
        } else if key == 'k' as u32 && modi == gdk::ModifierType::CONTROL_MASK {
            self.process_keyevent_move(-1);
            Inhibit(true)
        } else if key == 'c' as u32 && modi == gdk::ModifierType::CONTROL_MASK {
            self.process_keyevent_copy();
            Inhibit(true)
        } else if key == gdk::enums::key::Down {
            self.process_keyevent_move(1);
            Inhibit(true)
        } else if key == gdk::enums::key::Up {
            self.process_keyevent_move(-1);
            Inhibit(true)
        } else if let Some(ch) = gdk::keyval_to_unicode(key) {
            if ch.is_alphanumeric() {
                self.process_keyevent_char(ch);
            } else {
                trace!("Ignore char: {}", ch);
            }
            Inhibit(false)
        } else {
            Inhibit(false)
        }
    }

    fn reset_window(&mut self, send_clipboard: bool) {
        trace!("Resetting window: {}", send_clipboard);
        self.ctx.reset();
        self.status = Status::Initial;
        if send_clipboard {
            if let Err(error) = self.ctx.quicksend_from_clipboard() {
                warn!("Unable to get content from clipboard: {}", error);
            } else {
                self.status = Status::Default;
            }
        }
        self.ui.window.show();
        self.update_ui();
    }

    pub fn new(configpath: &std::path::Path) -> &'static thread::LocalKey<RefCell<Option<MinionsApp>>> {
        let config = Config::new(configpath);
        let global_config = config.partial(&["core"]).unwrap();

        let matcher = Matcher::new(
            &global_config.get_filename(&["statistic_file"]).unwrap(),
            &global_config.get::<String>(&["statistic_file_salt"]).unwrap()
            ).unwrap();

        let ctx = Context::new(&config);

        let app = MinionsApp {
            ui: MinionsUI::new(),
            ctx: ctx,
            status: Status::Initial,
            filter_timeout: std::time::Duration::from_millis(global_config.get::<u64>(&["filter_timeout"]).unwrap()),
            matcher: matcher,
        };
        app.update_ui();
        app.ui.window.hide();

        app.ui.window.connect_key_press_event(move |_, event| {
            APP.with(|app| {
                if let Some(ref mut app) = *app.borrow_mut() {
                    app.process_keyevent(event)
                } else { Inhibit(false) }
            })
        });

        app.ui.textentry.connect_changed(move |_| {
            glib::idle_add(move || {
                APP.with(|app| {
                    if let Some(ref mut app) = *app.borrow_mut() {
                        app.process_entry_text_changed()
                    }
                });
                Continue(false)
            });
        });

        unsafe {
            keybinder_init();
            let keys = global_config.get::<String>(&["shortcut_show"]).unwrap();
            if keys.len() > 0 {
                info!("Binding shortcut for show: {}", keys);
                let s = ffi::CString::new(keys).unwrap();
                keybinder_bind(s.as_ptr(), keybinder_callback_show, std::ptr::null_mut());
            } else {
                warn!("No shortcut defined for show");
            }

            let keys = global_config.get::<String>(&["shortcut_show_quicksend"]).unwrap();
            if keys.len() > 0 {
                info!("Binding shortcut for show_quicksend: {}", keys);
                let s = ffi::CString::new(keys).unwrap();
                keybinder_bind(s.as_ptr(), keybinder_callback_show_clipboard, std::ptr::null_mut());
            } else {
                warn!("No shortcut defined for show_quicksend");
            }
        }

        app.ui.window.connect_delete_event(move |_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        let configpath = configpath.to_path_buf();
        glib::source::unix_signal_add(1, move || {
            APP.with(|app| {
                let newconfig = Config::new(&configpath);
                if let Some(ref mut app) = *app.borrow_mut() {
                    info!("Received SIGHUP, reloading context");
                    app.ctx.reload(&newconfig);
                }
            });
            Continue(true)
        });

        APP.with(|g_app| *g_app.borrow_mut() = Some(app) );
        &APP
    }
}
