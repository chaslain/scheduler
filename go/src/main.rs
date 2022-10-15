extern crate telegram;

use telegram::BotBoy;
fn main() {
    let bot = BotBoy::new();
    
    bot.process_updates();
    
}
