/*
* @Author: BlahGeek
* @Date:   2017-04-20
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-13
*/

use std::error::Error;
use std::rc::Rc;
use mcore::item::Item;
use mcore::fuzzymatch::fuzzymatch;
use mcore::quicksend::quicksend;
use actions;


pub struct Context {
    /// Reference item for quick-send
    pub reference_item: Option<Rc<Item>>,
    /// Candidates items list
    pub list_items: Vec<Rc<Item>>,

    /// Stack of history items, init with empty stack
    /// Calling the last item's action would yields list_items
    history_items: Vec<Rc<Item>>,
}


impl Context {

    /// Create context with initial items
    pub fn new() -> Context {
        Context {
            reference_item: None,
            list_items: actions::get_actions().into_iter()
            .filter(|action| {
                action.accept_nothing() || action.accept_text()
            })
            .map(|action| {
                Rc::new(Item::new_action_item(action))
            }).collect(),
            history_items: Vec::new(),
        }
    }

    /// Filter list_items using fuzzymatch
    pub fn filter(&self, pattern: &str) -> Vec<Rc<Item>> {
        println!("filter: {:?}", pattern);
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
            action.accept_text()
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
            self.list_items = quicksend(data).into_iter().map(|x| Rc::new(x))
                                             .collect::<Vec<Rc<Item>>>();
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
            *self = Context::new();
            Ok(())
        }
    }
}

