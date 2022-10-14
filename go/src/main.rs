extern crate telegram;

use telegram::BotBoy;
fn main() {
    let bot = BotBoy::new();

    let mut response: Option<String> = None;
    let options = vec!("Yes".to_owned(), "No".to_owned());
    match bot.send_message_to_user_with_option_response(1846306122, &"Test".to_owned(), &options) {
        Ok(mut ok) => {
            response = Some(ok.text().unwrap());
        },
        Err(_) => {}
    };


    println!("{}", response.unwrap());
    

    
}
