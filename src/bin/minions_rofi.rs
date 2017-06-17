/*
* @Author: BlahGeek
* @Date:   2017-06-13
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-17
*/

extern crate minions;
extern crate env_logger;

fn main() {
    env_logger::init().unwrap();

    let mut app = minions::frontend_rofi::app::MinionsApp::new();
    app.run_loop();
}
