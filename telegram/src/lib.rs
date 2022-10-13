extern crate chatterbox;

use serde_json::from_str;
use std::env;

pub fn process(update: &String) {
    let token = env::var("BOT_TOKEN").unwrap();
}
