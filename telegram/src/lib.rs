extern crate chatterbox;
extern crate serde_json;

use reqwest::Client;
use reqwest::Result;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::env;

#[derive(Serialize, Deserialize)]
struct Video {
    file_id: String,
}
#[derive(Serialize, Deserialize)]
struct User {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct Chat {
    id: i32,

    #[serde(rename = "type")]
    _type: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct InlineQuery {

}

#[derive(Serialize, Deserialize)]
struct CallbackQuery {
    id: String,
    from: User,
    message: Message,
    data: String

}

struct Update {
    message: Option<Message>,
    callback_query: Option<CallbackQuery>
}

#[derive(Serialize, Deserialize)]
struct SendMessage {
    chat_id: i32,
    text: String,
    reply_markup: Option<InlineKeyboardMarkup>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    chat: Option<Chat>,
    from: Option<User>,
    text: Option<String>,
    video: Option<Video>,
}

struct Values {
    base_url: String,
    send_url: String,
}

#[derive(Serialize, Deserialize)]
struct InlineKeyboardMarkup {
    inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

#[derive(Serialize, Deserialize)]
struct InlineKeyboardButton {
    text: String,
    callback_data: String,
}

impl Values {
    pub fn new() -> Values {
        Values {
            base_url: String::from("https://api.telegram.org/bot"),
            send_url: String::from("sendMessage"),
        }
    }

    pub fn get_url_send(&self, token: &String) -> String {
        let mut result = self.base_url.to_owned();
        result.push_str(token);
        result.push('/');
        result.push_str(&self.send_url);

        result
    }
}

pub struct BotBoy {
    token: String,
    values: Values,
    client: Client,
}

impl BotBoy {
    pub fn new() -> BotBoy {
        BotBoy {
            token: env::var("BOT_TOKEN").unwrap(),
            values: Values::new(),
            client: Client::new(),
        }
    }

    pub fn send_message_to_user(
        &self,
        user_id: i32,
        message: &String,
    ) -> Result<reqwest::Response> {
        let message = SendMessage {
            chat_id: user_id,
            text: message.to_string(),
            reply_markup: None,
        };

        self.send_object(&self.values.get_url_send(&self.token), message)
    }

    pub fn send_message_to_user_with_option_response(
        &self,
        user_id: i32,
        message: &String,
        options: &Vec<String>,
    ) -> Result<reqwest::Response> {
        let mut items: Vec<InlineKeyboardButton> = Vec::new();
        for option in options {
            items.push(InlineKeyboardButton {
                text: option.to_owned(),
                callback_data: option.to_owned(),
            })
        }

        let mut items2: Vec<Vec<InlineKeyboardButton>> = Vec::new();
        items2.push(items);
        let message = SendMessage {
            chat_id: user_id,
            text: message.to_string(),
            reply_markup: Some(InlineKeyboardMarkup {
                inline_keyboard: items2,
            }),
        };

        self.send_object(&&self.values.get_url_send(&self.token), message)
    }

    fn send_object<T: Serialize>(&self, url: &String, object: T) -> Result<reqwest::Response>
    {
        println!("{}",url);
        let body = to_string(&object).unwrap();
        println!("{}", body);

        let req = self
            .client
            .post(url.as_str())
            .body(body)
            .header("Content-Type", "application/json")
            .build()
            .unwrap();

        for i in req.headers().keys() {
            println!("{}", req.headers().get(i).unwrap().to_str().unwrap());
        }

        self.client.execute(req)
    }
}
