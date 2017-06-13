/*
* @Author: BlahGeek
* @Date:   2017-06-13
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-06-13
*/

extern crate minions;

fn main() {
    let mut app = minions::frontend_rofi::app::MinionsApp::new();
    app.run_loop();
}
