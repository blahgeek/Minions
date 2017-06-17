pub mod mcore;
pub mod actions;

#[macro_use]
extern crate lazy_static;

#[cfg(feature="use-gtk")]
pub mod frontend_gtk;

pub mod frontend_rofi;

#[cfg(test)]
mod tests {
    use actions;
    #[test]
    fn it_works() {
        let actions = actions::get_actions();
        let items = actions[0].run_text("Hello world.");
        println!("Item len: {}", items.unwrap().len());
    }
}
