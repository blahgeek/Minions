pub mod mcore;
pub mod actions;

#[cfg(test)]
mod tests {
    use actions;
    #[test]
    fn it_works() {
        let actions = actions::get_actions();
        let items = actions[0].run_text("Hello world.");
        println!("Item len: {}", items.len());
        for item in &items {
            println!("{}", item);
        }
        for item in &items {
            println!("{}", item);
        }
    }
}
