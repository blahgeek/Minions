pub mod mcore;
pub mod actions;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;
extern crate toml;

#[cfg(feature="use-gtk")]
pub mod frontend_gtk;

pub mod frontend_rofi;
