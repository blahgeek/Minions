/*
* @Author: BlahGeek
* @Date:   2017-04-20
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-17
*/

use toml;

use std::error::Error;
use std::rc::Rc;
use mcore::action::{Action, ActionArg};
use mcore::item::{Item, ItemData};
use mcore::fuzzymatch::fuzzymatch;
use actions;


pub struct Context {
    /// Reference item for quick-send
    pub reference_item: Option<Rc<Item>>,
    /// Candidates items list
    pub list_items: Vec<Rc<Item>>,

    /// Stack of history items, init with empty stack
    /// Calling the last item's action would yields list_items
    history_items: Vec<Rc<Item>>,

    /// Cached all actions
    all_actions: Vec<Rc<Box<Action>>>,
}


impl Context {

    /// Create context with initial items
    pub fn new(config: toml::Value) -> Context {
        let mut ctx = Context {
            reference_item: None,
            list_items: Vec::new(),
            history_items: Vec::new(),
            all_actions: actions::get_actions(config),
        };
        ctx.reset();
        ctx
    }

    /// Reset context to initial state
    pub fn reset(&mut self) {
        self.reference_item = None;
        self.list_items = self.all_actions.iter()
            .filter(|action| {
                action.accept_nothing() || action.accept_text()
            })
            .map(|action| Item::new_action_item(action.clone()))
            .map(|item| Rc::new(item))
            .collect();
        self.list_items.sort_by_key(|item| item.priority );
        self.history_items = Vec::new();
    }

    /// Filter list_items using fuzzymatch
    pub fn filter(&self, pattern: &str) -> Vec<Rc<Item>> {
        trace!("filter: {:?}", pattern);
        let scores = self.list_items.iter().map(|item| {
            let search_str = if let Some(ref search_str) = item.search_str {
                search_str
            } else {
                &item.title
            };
            fuzzymatch(search_str, pattern, false)
        });
        let mut items_and_scores = self.list_items.clone().into_iter().zip(scores.into_iter())
            .collect::<Vec<(Rc<Item>, i32)>>();
        items_and_scores.sort_by_key(|item_and_score| -item_and_score.1);
        items_and_scores.into_iter()
            .filter(|item_and_score| item_and_score.1 > 0)
            .map(|item_and_score| item_and_score.0)
            .collect::<Vec<Rc<Item>>>()
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

    pub fn select(&mut self, item: Rc<Item>) -> Result<(), Box<Error>> {
        if !self.selectable(&item) {
            panic!("Item {} is not selectable", item);
        }
        if let Some(ref action) = item.action {
            self.list_items = action.run_arg(&item.action_arg)?
                              .into_iter()
                              .map(|x| Rc::new(x))
                              .collect::<Vec<Rc<Item>>>();
            self.list_items.sort_by_key(|item| item.priority );
        } else {
            panic!("Should not reach here");
        }
        self.history_items.push(item);
        self.reference_item = None;
        Ok(())
    }

    pub fn select_with_text(&mut self, item: Rc<Item>, text: &str) -> Result<(), Box<Error>> {
        if !self.selectable_with_text(&item) {
            panic!("Item {} is not selectable with text", &item);
        }
        if let Some(ref action) = item.action {
            self.list_items = action.run_text(text)?
                              .into_iter()
                              .map(|x| Rc::new(x))
                              .collect::<Vec<Rc<Item>>>();
            self.list_items.sort_by_key(|item| item.priority );
        } else {
            panic!("Should not reach here");
        }
        self.history_items.push(item);
        self.reference_item = None;
        Ok(())
    }

    pub fn quicksend_able(&self, item: &Item) -> bool {
        self.reference_item.is_none() && item.data.is_some()
    }

    pub fn quicksend(&mut self, item: Rc<Item>) -> Result<(), Box<Error>> {
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
                        Rc::new(item)
                    })
                    .collect()
                },
                &ItemData::Path(ref path) => {
                    self.all_actions.iter()
                    .filter(|action| action.accept_path())
                    .map(|action| {
                        let mut item = Item::new_action_item(action.clone());
                        item.action_arg = ActionArg::Path(path.clone());
                        Rc::new(item)
                    })
                    .collect()
                },
            };
            self.list_items.sort_by_key(|item| item.priority );
        } else {
            panic!("Should not reach here");
        }
        self.reference_item = Some(item);
        Ok(())
    }

    pub fn back(&mut self) -> Result<(), Box<Error>> {
        if let Some(action_item) = self.history_items.pop() {
            self.select(action_item)
        } else {
            self.reset();
            Ok(())
        }
    }
}

