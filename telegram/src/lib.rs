pub extern crate chatterbox;

use chatterbox::accept_incoming_message;
use chatterbox::Coorespondance;
use chatterbox::FlowStatus;
use chatterbox::Message as ChatterMessage;
use chatterbox::OptionType;
use reqwest::Client;
use reqwest::Result;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::env;

#[derive(Serialize, Deserialize)]
struct File {
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
struct GetChat {
    chat_id: String,
}
#[derive(Serialize, Deserialize)]
struct CallbackQuery {
    id: String,
    from: User,
    message: Message,
    data: String,
}

#[derive(Serialize, Deserialize)]
pub struct Update {
    update_id: i64,
    message: Option<Message>,
    callback_query: Option<CallbackQuery>,
}

#[derive(Serialize, Deserialize)]
struct SendMessage {
    chat_id: i64,
    text: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    reply_markup: Option<InlineKeyboardMarkup>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum SendMedia {
    Photo { chat_id: String, photo: String },
    Video { chat_id: String, video: String },
    Audio { chat_id: String, audio: String },
    Document { chat_id: String, document: String },
    Voice { chat_id: String, voice: String },
}
#[derive(Serialize, Deserialize)]
struct Message {
    chat: Option<Chat>,
    from: Option<User>,
    text: Option<String>,
    video: Option<Vec<File>>,
    document: Option<Vec<File>>,
    photo: Option<Vec<File>>,
    entities: Option<Vec<Entity>>,
    voice: Option<File>,
}

#[derive(Serialize, Deserialize)]
struct Entity {
    offset: i32,
    length: i32,
    #[serde(rename = "type")]
    _type: String,
}

#[derive(Serialize, Deserialize)]
struct BareResponse {
    ok: bool,
}

struct Values {
    base_url: String,
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
        }
    }

    pub fn get_url_send(&self, token: &String) -> String {
        let mut result = self.base_url.to_owned();
        result.push_str(token);
        result.push('/');
        result.push_str("sendMessage");

        result
    }

    pub fn get_url_updates(&self, token: &String) -> String {
        format!("{}{}/getUpdates", self.base_url, token)
    }

    pub fn get_url_chat(&self, token: &String) -> String {
        format!("{}{}/getChat", self.base_url, token)
    }

    pub fn get_url_photo(&self, token: &String) -> String {
        format!("{}{}/sendPhoto", self.base_url, token)
    }

    pub fn get_url_video(&self, token: &String) -> String {
        format!("{}{}/sendVideo", self.base_url, token)
    }

    pub fn get_url_document(&self, token: &String) -> String {
        format!("{}{}/sendDocument", self.base_url, token)
    }

    pub fn get_url_audio(&self, token: &String) -> String {
        format!("{}{}/sendAudio", self.base_url, token)
    }

    pub fn get_url_voice(&self, token: &String) -> String {
        format!("{}{}/sendVoice", self.base_url, token)
    }
}

#[derive(Serialize, Deserialize)]
struct Response<T> {
    ok: bool,
    result: T,
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

    pub fn send_media_to_user(&self, user_id: i64, message: ChatterMessage) {
        match message {
            ChatterMessage::Message(text) => {
                _ = self.send_message_to_user(user_id, &text);
            }
            ChatterMessage::Photo(id) => match self.send_photo(&user_id.to_string(), &id) {
                Ok(()) => {}
                Err(_e) => {}
            },
            ChatterMessage::Audio(id) => match self.send_audio(&user_id.to_string(), &id) {
                Ok(()) => {}
                Err(_e) => {}
            },
            ChatterMessage::Video(id) => match self.send_video(&user_id.to_string(), &id) {
                Ok(()) => {}
                Err(_e) => {}
            },
            ChatterMessage::Document(id) => match self.send_document(&user_id.to_string(), &id) {
                Ok(()) => {}
                Err(_e) => {}
            },
            ChatterMessage::Voice(id) => match self.send_voice(&user_id.to_string(), &id) {
                Ok(()) => {}
                Err(_e) => {}
            },
        }
    }

    pub fn send_message_to_user(
        &self,
        user_id: i64,
        message: &String,
    ) -> Result<reqwest::Response> {
        let send_message = SendMessage {
            chat_id: user_id,
            text: message.to_string(),
            reply_markup: None,
        };

        self.send_object(&self.values.get_url_send(&self.token), send_message)
    }

    pub fn send_message_to_user_with_yesno(
        &self,
        user_id: i64,
        message: &String,
    ) -> Result<reqwest::Response> {
        let options = vec![vec!["Yes".to_owned(), "No".to_owned()]];

        self.send_message_to_user_with_option_response(user_id, message, &options)
    }

    pub fn send_message_to_user_with_option_response(
        &self,
        user_id: i64,
        message: &String,
        options: &Vec<Vec<String>>,
    ) -> Result<reqwest::Response> {
        let mut items: Vec<Vec<InlineKeyboardButton>> = Vec::new();

        let mut index = 0;

        let mut i = 0;
        while i < options.len() {
            items.push(Vec::new());
            i += 1;
        }

        for option in options {
            for button in option {
                items.get_mut(index).unwrap().push(InlineKeyboardButton {
                    text: button.to_owned(),
                    callback_data: button.to_owned(),
                });
            }

            index += 1;
        }

        let message = SendMessage {
            chat_id: user_id,
            text: message.to_string(),
            reply_markup: Some(InlineKeyboardMarkup {
                inline_keyboard: items,
            }),
        };

        self.send_object(&&self.values.get_url_send(&self.token), message)
    }

    pub fn send_object<T: Serialize>(&self, url: &String, object: T) -> Result<reqwest::Response> {
        let body = to_string(&object).unwrap();

        let req = self
            .client
            .post(url.as_str())
            .body(body)
            .header("Content-Type", "application/json")
            .build()
            .unwrap();
        
        println!("sending request...");

        let res = self.client.execute(req);
        println!("done");
        return res;
    }

    pub fn process_updates(&self) {
        match self.get_updates_manual() {
            Ok(text) => {
                let update_id = self.process_update_from_string(&text);

                match update_id {
                    Some(id) => {
                        self.get_updates_option(id);
                    }
                    None => {}
                }
            }
            Err(_) => {
                panic!("so here's the thing....");
            }
        }
    }
    pub fn process_single_update_from_string(&self, update_string: &String) {
        match serde_json::from_str(&update_string) {
            Ok(up) => {
                let mut wrapper: Vec<Update> = Vec::new();
                wrapper.push(up);

                self.process_updates_vec(wrapper);
            },

            Err(e) => {
                println!("{}", e);
            }
        }
    }

    pub fn process_update_from_string(&self, update_string: &String) -> Option<i64> {
        let updates = match self.get_updates(update_string) {
            Ok(data) => data,
            Err(_) => panic!("problem!!"),
        };

        self.process_updates_vec(updates)
    }

    pub fn process_updates_vec(&self, updates: Vec<Update>) -> Option<i64>
    {
        println!("inside process updates vec");
        if updates.last().is_some() {
            let result = Some(updates.last().unwrap().update_id);

            for i in updates {
                if i.message.is_some() {
                    self.handle_message_update(i)
                } else if i.callback_query.is_some() {
                    self.handle_query_update(i);
                } else {
                    panic!("Unhandled update type");
                }
            }

            return result;
        } else {
            return None;
        }
    }

    pub fn get_token(&self) -> String {
        self.token.to_owned()
    }

    fn handle_query_update(&self, i: Update) {
        let query = i.callback_query.unwrap();
        let (text, chat_id) = get_string_from_query(query);
        self.update_from_message(chat_id, &ChatterMessage::Message(text));
    }

    fn handle_message_update(&self, i: Update) {
        let message = i.message.unwrap();
        let (text, chat_id) = self.get_string_from_message(&message);
        self.update_from_message(chat_id, &text);
    }

    fn update_from_message(&self, chat_id: i64, message: &ChatterMessage) {
        let flow_status = accept_incoming_message(&chat_id.to_string(), message);

        match flow_status {
            FlowStatus::Cancelled => {
                let _ = self.send_message_to_user(
                    chat_id,
                    &"Cancelled.".to_owned(),
                );
            }
            FlowStatus::Done => {
                _ =
                    self.send_message_to_user(chat_id, &"Your messages are scheduled! Ensure I (or the bot you're using) is added to the chat and has the ability to send messages.".to_owned());
            }
            FlowStatus::DoneWithMessage(message) => {
                _ = self.send_message_to_user(chat_id, &message);
            }
            FlowStatus::Step(coorespondance) => {
                self.use_coorespondance(chat_id, coorespondance);
            }
            FlowStatus::Error {
                message,
                desired_value: _,
            } => {
                _ = self.send_message_to_user(chat_id, &message);
            }
            FlowStatus::Info(message) => {
                _ = self.send_message_to_user(chat_id, &message);
            }
            FlowStatus::Media(message) => {
                _ = self.send_media_to_user(chat_id, message);
            }
        }
    }

    fn use_coorespondance(&self, chat_id: i64, coorespondance: Coorespondance) {
        let _response: Option<::core::result::Result<::reqwest::Response, ::reqwest::Error>> =
            match coorespondance.option_type {
                OptionType::Date => {
                    Some(self.send_message_to_user(chat_id, &coorespondance.message))
                }
                OptionType::Media => {
                    Some(self.send_message_to_user(chat_id, &coorespondance.message))
                }
                OptionType::YesNo => {
                    Some(self.send_message_to_user_with_yesno(chat_id, &coorespondance.message))
                }
                OptionType::Time => {
                    Some(self.send_message_to_user(chat_id, &coorespondance.message))
                }
                OptionType::Options(options) => {
                    Some(self.send_message_to_user_with_option_response(
                        chat_id,
                        &coorespondance.message,
                        &options,
                    ))
                }
                OptionType::None => None,
            };
    }

    fn get_updates(&self, input: &String) -> core::result::Result<Vec<Update>, ()> {
        match self.get_updates_webhook(input) {
            Ok(obj) => return Ok(obj),
            Err(_) => {} // nothing yet, still have another method to try...
        }

        match self.get_updates_from_manual(input) {
            Ok(obj) => return Ok(obj.result),
            Err(_) => return Err(()),
        }
    }

    fn get_updates_webhook(&self, input: &String) -> core::result::Result<Vec<Update>, ()> {
        match serde_json::from_str::<Vec<Update>>(&input) {
            Ok(obj) => Ok(obj),
            Err(_) => Err(()),
        }
    }

    fn get_updates_from_manual(
        &self,
        input: &String,
    ) -> core::result::Result<Response<Vec<Update>>, ()> {
        match serde_json::from_str::<Response<Vec<Update>>>(&input) {
            Ok(obj) => Ok(obj),
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    fn get_updates_manual(&self) -> core::result::Result<String, ()> {
        let url = self.values.get_url_updates(&self.token);

        match self.client.get(&url).send() {
            Ok(mut response) => {
                let text = response.text().unwrap();
                Ok(text)
            }
            Err(_) => Err(()),
        }
    }

    fn get_updates_option(&self, id: i64) {
        let url = self.values.get_url_updates(&self.token);

        let _ = self
            .client
            .get(&url)
            .query(&[("offset", &(id + 1).to_string())])
            .send();
    }

    pub fn get_chat(&self, chat_id: &String) -> String {
        let url = self.values.get_url_chat(&self.token);

        let response = self.send_object(
            &url,
            GetChat {
                chat_id: chat_id.to_owned(),
            },
        );

        let update: BareResponse =
            serde_json::from_str(&response.unwrap().text().unwrap()).unwrap();

        if update.ok {
            chat_id.to_owned()
        } else {
            "INVALID".to_owned()
        }
    }

    fn send_media(&self, media: SendMedia, url: &String) -> ::core::result::Result<(), String> {
        match self.send_object(&url, media) {
            Ok(mut response) => {
                let text = response.text().unwrap();

                let parsed = serde_json::from_str::<BareResponse>(&text).unwrap();

                if parsed.ok {
                    Ok(())
                } else {
                    Err("Bad request".to_owned())
                }
            }
            Err(_) => Err("Request Failed".to_owned()),
        }
    }

    pub fn send_document(
        &self,
        chat_id: &String,
        document: &String,
    ) -> ::core::result::Result<(), String> {
        let url = self.values.get_url_document(&self.token);

        let media = SendMedia::Document {
            chat_id: chat_id.to_owned(),
            document: document.to_owned(),
        };

        self.send_media(media, &url)
    }

    pub fn send_audio(
        &self,
        chat_id: &String,
        audio: &String,
    ) -> ::core::result::Result<(), String> {
        let url = self.values.get_url_audio(&self.token);

        let media = SendMedia::Audio {
            chat_id: chat_id.to_owned(),
            audio: audio.to_owned(),
        };

        self.send_media(media, &url)
    }

    pub fn send_voice(
        &self,
        chat_id: &String,
        voice: &String,
    ) -> ::core::result::Result<(), String> {
        let url = self.values.get_url_voice(&self.token);

        let media = SendMedia::Voice {
            chat_id: chat_id.to_owned(),
            voice: voice.to_owned(),
        };

        self.send_media(media, &url)
    }

    pub fn send_video(
        &self,
        chat_id: &String,
        video: &String,
    ) -> ::core::result::Result<(), String> {
        let url = self.values.get_url_video(&self.token);

        let media = SendMedia::Video {
            chat_id: chat_id.to_owned(),
            video: video.to_owned(),
        };

        self.send_media(media, &url)
    }

    pub fn send_photo(
        &self,
        chat_id: &String,
        photo: &String,
    ) -> ::core::result::Result<(), String> {
        let url = self.values.get_url_photo(&self.token);

        let media = SendMedia::Photo {
            chat_id: chat_id.to_owned(),
            photo: photo.to_owned(),
        };

        self.send_media(media, &url)
    }

    fn get_string_from_message(&self, message: &Message) -> (ChatterMessage, i64) {
        if message.entities.is_some() {
            let entities = message.entities.as_ref().unwrap();
            let entity = entities.get(0).unwrap();
            match entity._type.as_str() {
                "mention" => {
                    let offset = entity.offset;
                    let length = entity.length;
                    let text = message.text.as_ref().unwrap();
                    let result = &text[offset as usize..length as usize];
                    (
                        ChatterMessage::Message(self.get_chat(&result.to_string())),
                        message.chat.as_ref().unwrap().id,
                    )
                }
                _ => {
                    // same as normal message logic
                    (
                        ChatterMessage::Message(message.text.as_ref().unwrap().to_owned()),
                        message.chat.as_ref().unwrap().id,
                    )
                }
            }
        } else if message.text.is_some() {
            (
                ChatterMessage::Message(message.text.as_ref().unwrap().to_owned()),
                message.chat.as_ref().unwrap().id,
            )
        } else if message.video.is_some() {
            (
                ChatterMessage::Video(
                    message
                        .video
                        .as_ref()
                        .unwrap()
                        .get(0)
                        .unwrap()
                        .file_id
                        .to_owned(),
                ),
                message.chat.as_ref().unwrap().id,
            )
        } else if message.document.is_some() {
            (
                ChatterMessage::Document(
                    message
                        .document
                        .as_ref()
                        .unwrap()
                        .get(0)
                        .unwrap()
                        .file_id
                        .to_owned(),
                ),
                message.chat.as_ref().unwrap().id,
            )
        } else if message.photo.is_some() {
            (
                ChatterMessage::Photo(
                    message
                        .photo
                        .as_ref()
                        .unwrap()
                        .get(0)
                        .unwrap()
                        .file_id
                        .to_owned(),
                ),
                message.chat.as_ref().unwrap().id,
            )
        } else if message.voice.is_some() {
            (
                ChatterMessage::Voice(message.voice.as_ref().unwrap().file_id.to_owned()),
                message.chat.as_ref().unwrap().id,
            )
        } else {
            panic!("Unsupported message type");
        }
    }
}

fn get_string_from_query(query: CallbackQuery) -> (String, i64) {
    (query.data, query.message.chat.unwrap().id)
}
