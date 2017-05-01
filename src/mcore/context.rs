/*
* @Author: BlahGeek
* @Date:   2017-04-20
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-05-01
*/

use std::error::Error;
use mcore::item::Item;
use mcore::fuzzymatch::fuzzymatch;
use mcore::quicksend::quicksend;
use actions;


pub struct Context {
    /// Reference item for quick-send
    pub reference_item: Option<Item>,
    /// Candidates items list
    pub list_items: Vec<Item>,

    /// Stack of history items, init with empty stack
    /// Calling the last item's action would yields list_items
    history_items: Vec<Item>,
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
                Item::new_action_item(action)
            }).collect(),
            history_items: Vec::new(),
        }
    }

    /// Filter list_items using fuzzymatch
    pub fn filter(&self, pattern: &str) -> Vec<&Item> {
        println!("filter: {:?}", pattern);
        let scores = self.list_items.iter().map(|item| {
            let search_str = if let Some(ref search_str) = item.search_str {
                search_str
            } else {
                &item.title
            };
            fuzzymatch(search_str, pattern, false)
        });
        let mut items_and_scores = self.list_items.iter().zip(scores.into_iter())
            .collect::<Vec<(&Item, i32)>>();
        items_and_scores.sort_by_key(|item_and_score| -item_and_score.1);
        items_and_scores.into_iter()
            .filter(|item_and_score| item_and_score.1 > 0)
            .map(|item_and_score| item_and_score.0)
            .collect::<Vec<&Item>>()
    }

    /// Get item from context, destroy list_items
    pub fn get_item(&mut self, idx: usize) -> Item {
        while self.list_items.len() > idx + 1 {
            self.list_items.pop();
        }
        let ret = self.list_items.pop();
        self.list_items.clear();
        ret.unwrap()
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

    pub fn select(&mut self, item: Item) -> Result<(), Box<Error>> {
        if !self.selectable(&item) {
            panic!("Item {} is not selectable", item);
        }
        if let Some(ref action) = item.action {
            self.list_items = action.run_arg(&item.action_arg)?;
        } else {
            panic!("Should not reach here");
        }
        self.history_items.push(item);
        self.reference_item = None;
        Ok(())
    }

    pub fn select_with_text(&mut self, item: Item, text: &str) -> Result<(), Box<Error>> {
        if !self.selectable_with_text(&item) {
            panic!("Item {} is not selectable with text", item);
        }
        if let Some(ref action) = item.action {
            self.list_items = action.run_text(text)?;
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

    pub fn quicksend(&mut self, item: Item) -> Result<(), Box<Error>> {
        if !self.quicksend_able(&item) {
            panic!("Item {} is not quicksend_able", item);
        }
        if let Some(ref data) = item.data {
            self.list_items = quicksend(data);
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

