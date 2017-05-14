extern crate gtk;
extern crate gdk;

pub mod mcore;
pub mod actions;
pub mod frontend;

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
