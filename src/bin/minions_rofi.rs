/*
* @Author: BlahGeek
* @Date:   2017-06-13
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-17
*/

extern crate minions;
extern crate env_logger;
extern crate toml;

fn main() {
    env_logger::init().unwrap();

    let config = include_str!("../../config/example.toml");
    let config = config.parse::<toml::Value>().unwrap();

    let mut app = minions::frontend_rofi::app::MinionsApp::new(config);
    app.run_loop();
}
