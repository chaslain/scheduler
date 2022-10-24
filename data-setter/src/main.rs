use telegram::BotBoy;
use serde::Serialize;

#[derive(Serialize)]
struct SetWebHook {
    url: String
}

fn main() {
    let bot = BotBoy::new();

    let url = format!("https://api.telegram.org/bot{}/setWebhook", bot.get_token());

    let object = SetWebHook {
        url: "https://3cddtsqpk8.execute-api.us-east-2.amazonaws.com/default/recurring-message-scheduler".to_owned()
    };
    
    match bot.send_object(&url, object) {
        Ok(mut resp) => {
            println!("{}", resp.text().unwrap())
        },
        Err(_) => {}
    }
}
