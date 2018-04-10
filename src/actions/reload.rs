extern crate nix;

use std::sync::Arc;

use mcore::action::{Action, ActionResult};
use mcore::item::{Item, Icon};
use mcore::config::Config;
use mcore::errors::*;


struct ReloadAction {}

impl Action for ReloadAction {
    fn runnable_bare(&self) -> bool { true }
    fn run_bare(&self) -> ActionResult {
        nix::sys::signal::kill(0, nix::sys::signal::SIGHUP)
            .map_err(|e| Error::with_chain(e, "Failed to send SIGHUP to myself"))?;
        Ok(Vec::new())
    }
}

pub fn get(_: &Config) -> Item {
    Item {
        title: "Reload All Actions".into(),
        subtitle: Some("Equivalent to `kill -HUP 0`".into()),
        badge: Some("Minions".into()),
        priority: 100,
        icon: Some(Icon::FontAwesome("cog".into())),
        action: Some(Arc::new(ReloadAction{})),
        .. Item::default()
    }
}
