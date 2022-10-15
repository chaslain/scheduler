extern crate chatterbox;
extern crate serde_json;

use chatterbox::Coorespondance;
use chatterbox::FlowStatus;
use chatterbox::OptionType;
use reqwest::Client;
use reqwest::Result;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::env;
use chatterbox::accept_incoming_message;

#[derive(Serialize, Deserialize)]
struct Video {
    file_id: String,
}
#[derive(Serialize, Deserialize)]
struct User {
    id: i64,
}

#[derive(Serialize, Deserialize)]
struct Chat {
    id: i64,

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

#[derive(Serialize, Deserialize)]
struct Update {
    update_id: i64,
    message: Option<Message>,
    callback_query: Option<CallbackQuery>
}

#[derive(Serialize, Deserialize)]
struct SendMessage {
    chat_id: i64,
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

    pub fn get_url_updates(&self, token: &String) -> String {
        format!("{}{}/getUpdates", self.base_url, token)
    }
}

#[derive(Serialize, Deserialize)]
struct Response<T> {
    ok: bool,
    result: T
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
        user_id: i64,
        message: &String,
    ) -> Result<reqwest::Response> {
        let message = SendMessage {
            chat_id: user_id,
            text: message.to_string(),
            reply_markup: None,
        };

        self.send_object(&self.values.get_url_send(&self.token), message)
    }

    pub fn send_message_to_user_with_yesno(
        &self,
        user_id: i64,
        message: &String
    ) -> Result<reqwest::Response>
    {
        let options = vec!["Yes".to_owned(), "No".to_owned()];

        self.send_message_to_user_with_option_response(user_id, message, &options)
    }

    pub fn send_message_to_user_with_option_response(
        &self,
        user_id: i64,
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

    pub fn process_updates(&self) {
        match self.get_updates_manual() {
            Ok(text) => {
               self.process_update_from_string(&text);
            },
            Err(_) => {
                panic!("so here's the thing....");
            }
        }
    }

    pub fn process_update_from_string(&self, update_string: &String)
    {
        let updates = self.get_updates(update_string).unwrap();

        for i in updates {
            if i.message.is_some() {
                let message = i.message.unwrap();
                let chat_id = message.chat.unwrap().id;
                let s_chat_id = chat_id.to_string();
                let flow_status = accept_incoming_message(&s_chat_id, &message.text.unwrap());

                match flow_status {
                    FlowStatus::Done => {
                        let _ = self.send_message_to_user(chat_id, &"Your messages are scheduled!".to_owned());
                    },
                    FlowStatus::Step(coorespondance) => {
                        self.use_coorespondance(chat_id, coorespondance);
                    },
                    FlowStatus::Error { message, desired_value: _ } => {
                        let _ = self.send_message_to_user(chat_id, &message);
                    }
                }
            }
        }
    }

    fn use_coorespondance(&self, chat_id: i64, coorespondance: Coorespondance) {
        let _ = match coorespondance.option_type {
            OptionType::Date => self.send_message_to_user(chat_id, &coorespondance.message),
            OptionType::Media => self.send_message_to_user(chat_id, &coorespondance.message),
            OptionType::YesNo => self.send_message_to_user_with_yesno(chat_id, &coorespondance.message),
            OptionType::Time => self.send_message_to_user(chat_id, &coorespondance.message),
            OptionType::Options(options) => self.send_message_to_user_with_option_response(chat_id, &coorespondance.message, &options)
        };
    }

    fn get_updates(&self, input: &String) -> core::result::Result<Vec<Update>,()>
    {
        match self.get_updates_webhook(input) {
            Ok(obj) => return Ok(obj),
            Err(_) => {} // nothing yet, still have another method to try...
        }

        match self.get_updates_from_manual(input) {
            Ok(obj) => return Ok(obj.result),
            Err(_) => return Err(())
        }
    }

    fn get_updates_webhook(&self, input: &String) -> core::result::Result<Vec<Update>,()> {
        match serde_json::from_str::<Vec<Update>>(&input) {
            Ok(obj) => Ok(obj),
            Err(_) => Err(())
        }
    }

    fn get_updates_from_manual(&self, input: &String) -> core::result::Result<Response<Vec<Update>>, ()> {
        match serde_json::from_str::<Response<Vec<Update>>>(&input) {
            Ok(obj) => Ok(obj),
            Err(_) => Err(())
        }
    }

    fn get_updates_manual(&self) -> core::result::Result<String, ()> {
        let url = self.values.get_url_updates(&self.token);

        println!("{}", url);
        match  self.client.get(&url).send() {
            Ok(mut response) => {
                let text = response.text().unwrap();
                println!("{}", text);
                Ok(text)
            },
            Err(_) => Err(())
        }
    }
}
