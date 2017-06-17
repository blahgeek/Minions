pub mod mcore;
pub mod actions;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[cfg(feature="use-gtk")]
pub mod frontend_gtk;

pub mod frontend_rofi;
