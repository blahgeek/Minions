/*
* @Author: BlahGeek
* @Date:   2017-08-10
* @Last Modified by:   BlahGeek
* @Last Modified time: 2017-08-10
*/

#[derive(Deserialize)]
pub struct GlobalConfig {
    pub filter_timeout: u32,
    pub shortcut_show: Option<String>,
    pub shortcut_show_quicksend: Option<String>,
    pub history_file_salt: String,
}
