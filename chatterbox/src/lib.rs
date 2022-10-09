extern crate event_manager;

use chrono::Datelike;
use chrono::NaiveDateTime;
use event_manager::create_schedule;
use event_manager::Config;
use event_manager::Message;
use event_manager::Schedule as emSchedule;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::fs::remove_file;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub enum FlowStatus {
    Done,
    Step(Coorespondance),
    Error {
        message: String,
        desired_value: DesiredValue,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Schedule {
    Daily,
    Weekly,
    Biweekly,
    Monthly,
    Yearly,
}

#[derive(Serialize, Deserialize)]
pub enum DesiredValue {
    Message,
    Frequency,
    StartDate,
    StartTime,
    Chat,
    HasToken,
    Token,
}

pub enum UserInput {
    Message(String),
    Frequency(Schedule),
    Time(String),
    Date(String),
    Media(String),
    YesNo(bool),
    Cancel,
}

#[derive(Serialize, Deserialize)]
struct ConfigInProgress {
    pub schedule: Option<Schedule>,
    pub desired_value: DesiredValue,
    pub first_execution_date: Option<String>,
    pub first_execution_time: Option<String>,
    pub chat_id: Option<String>,
    pub message: Option<Message>,
    pub has_token: Option<bool>,
    pub token: Option<String>,
}

impl ConfigInProgress {
    pub fn move_to_next_step(&mut self) {
        self.desired_value = match self.desired_value {
            DesiredValue::Message => DesiredValue::Frequency,
            DesiredValue::Frequency => DesiredValue::StartDate,
            DesiredValue::StartDate => DesiredValue::StartTime,
            DesiredValue::StartTime => DesiredValue::Chat,
            DesiredValue::Chat => DesiredValue::HasToken,
            DesiredValue::HasToken => DesiredValue::Token,
            DesiredValue::Token => DesiredValue::Message,
        }
    }
}

pub enum OptionType {
    Options(Vec<String>),
    Media,
    Time,
    Date,
    YesNo,
}

pub struct Coorespondance {
    pub option_type: OptionType,
    pub message: String,
}

pub fn process_incoming_message(u_id: &String, message: UserInput) {
    let mut state = get_state(&u_id);

    match message {
        UserInput::Cancel => delete_state(u_id),
        _ => {
            match state.desired_value {
                DesiredValue::Message => state = process_desired_message(state, message),
                DesiredValue::Frequency => state = process_frequency(state, message),
                DesiredValue::StartDate => state = process_date(state, message),
                DesiredValue::StartTime => state = process_time(state, message),
                DesiredValue::Chat => state = process_chat(state, message),
                DesiredValue::HasToken => state = process_has_token(state, message, u_id),
                DesiredValue::Token => state = process_token(state, message, u_id),
            }

            state.move_to_next_step();

            save_state(u_id, state);
        }
    }
}

fn process_desired_message(
    mut config_in_progress: ConfigInProgress,
    message: UserInput,
) -> ConfigInProgress {
    match message {
        UserInput::Message(desired_message) => {
            config_in_progress.message = Some(Message::Message(desired_message));
        }
        UserInput::Media(id) => config_in_progress.message = Some(Message::Media(id)),
        _ => panic!("Unsupported Input type"),
    };

    config_in_progress
}

fn process_frequency(
    mut config_in_progress: ConfigInProgress,
    message: UserInput,
) -> ConfigInProgress {
    match message {
        UserInput::Frequency(frequency) => config_in_progress.schedule = Some(frequency),
        _ => panic!("Unsupported Input type"),
    }

    config_in_progress
}

fn process_date(mut config_in_progress: ConfigInProgress, message: UserInput) -> ConfigInProgress {
    match message {
        UserInput::Date(date) => config_in_progress.first_execution_date = Some(date),
        _ => panic!("Unsupported Input type"),
    }

    config_in_progress
}

fn process_time(mut config_in_progress: ConfigInProgress, message: UserInput) -> ConfigInProgress {
    match message {
        UserInput::Time(time) => config_in_progress.first_execution_time = Some(time),
        _ => panic!("Unsupported Input type"),
    }

    config_in_progress
}

fn process_chat(mut confing_in_progress: ConfigInProgress, message: UserInput) -> ConfigInProgress {
    match message {
        UserInput::Message(string) => confing_in_progress.chat_id = Some(string),
        _ => panic!("Unsupported Input Type"),
    }

    confing_in_progress
}

fn process_has_token(
    mut config_in_progress: ConfigInProgress,
    message: UserInput,
    u_id: &String,
) -> ConfigInProgress {
    match message {
        UserInput::YesNo(answer) => {
            config_in_progress.has_token = Some(answer);

            if !answer {
                // flow is now done!
                config_in_progress = close(u_id, config_in_progress);
            }
        }

        _ => panic!("Unsupported Input Type"),
    }

    if !config_in_progress.has_token.unwrap() {
        close(u_id, config_in_progress)
    } else {
        config_in_progress
    }
}

fn process_token(
    mut config_in_progress: ConfigInProgress,
    message: UserInput,
    u_id: &String,
) -> ConfigInProgress {
    match message {
        UserInput::Message(token) => config_in_progress.token = Some(token),
        _ => panic!("Unsupported Input Type"),
    }

    close(u_id, config_in_progress)
}

fn close(u_id: &String, config_in_progress: ConfigInProgress) -> ConfigInProgress {
    delete_state(u_id);

    let config = Config {
        chat_id: config_in_progress.chat_id.to_owned().unwrap(),
        message: config_in_progress.message.to_owned().unwrap(),
    };

    match config_in_progress.schedule.to_owned().unwrap() {
        Schedule::Daily => {
            let schedule = emSchedule::Daily {
                time: config_in_progress.first_execution_time.to_owned().unwrap(),
            };

            let _ = create_schedule(u_id, config, schedule);
        }
        Schedule::Weekly => {
            let datetime = NaiveDateTime::parse_from_str(
                &config_in_progress.first_execution_date.to_owned().unwrap(),
                "%Y-%m-%d",
            )
            .unwrap();

            let schedule = emSchedule::Weekly {
                weekday: datetime.weekday(),
                time: config_in_progress.first_execution_time.to_owned().unwrap(),
            };

            let _ = create_schedule(u_id, config, schedule);
        }
        Schedule::Biweekly => {
            let datetime = NaiveDateTime::parse_from_str(
                &config_in_progress.first_execution_date.to_owned().unwrap(),
                "%Y-%m-%d",
            )
            .unwrap();

            let schedule = emSchedule::BiWeekly {
                weekday: datetime.weekday(),
                time: config_in_progress.first_execution_time.to_owned().unwrap(),
                odd: datetime.iso_week().week() % 2 == 0,
            };

            let _ = create_schedule(u_id, config, schedule);
        }
        Schedule::Monthly => {
            let datetime = NaiveDateTime::parse_from_str(
                &config_in_progress.first_execution_date.to_owned().unwrap(),
                "%Y-%m-%d",
            )
            .unwrap();

            let schedule = emSchedule::Monthly {
                day: datetime.day() as i32,
                time: config_in_progress.first_execution_time.to_owned().unwrap(),
            };

            let _ = create_schedule(u_id, config, schedule);
        }
        Schedule::Yearly => {
            let datetime = NaiveDateTime::parse_from_str(
                &config_in_progress.first_execution_date.to_owned().unwrap(),
                "%Y-%m-%d",
            )
            .unwrap();

            let schedule = emSchedule::Monthly {
                day: datetime.day() as i32,
                time: config_in_progress.first_execution_time.to_owned().unwrap(),
            };

            let _ = create_schedule(u_id, config, schedule);
        }
    }

    config_in_progress
}

pub fn get_message(u_id: &String) -> Coorespondance {
    let state = get_state(u_id);

    match state.desired_value {
        DesiredValue::Message => Coorespondance {
            message: "Send the message or media you'd like sent.".to_string(),
            option_type: OptionType::Media,
        },
        DesiredValue::Frequency => Coorespondance {
            message: "How often would you like this sent?".to_string(),
            option_type: OptionType::Options(vec![
                "Daily".to_owned(),
                "Weekly".to_owned(),
                "Biweekly".to_owned(),
                "Monthly".to_owned(),
                "Yearly".to_owned(),
            ]),
        },
        DesiredValue::StartDate => Coorespondance {
            message:
                "When should the first message be sent? (Future dates will be based off of this.)"
                    .to_string(),
            option_type: OptionType::Date,
        },
        DesiredValue::StartTime => Coorespondance {
            message: "What time should the first message be sent?".to_string(),
            option_type: OptionType::Time,
        },
        DesiredValue::Chat => Coorespondance {
            message: "Please link the chat or channel where this message will be posted."
                .to_string(),
            option_type: OptionType::Media,
        },
        DesiredValue::HasToken => Coorespondance {
            message: "Would you like this sent by your own custom bot?".to_string(),
            option_type: OptionType::YesNo,
        },
        DesiredValue::Token => Coorespondance {
            message: "Please provide the bot token.".to_string(),
            option_type: OptionType::Media,
        },
    }
}

fn get_state(u_id: &String) -> ConfigInProgress {
    let file_name: String = format!("in_progress/{}", u_id);

    let path = Path::new(&file_name);

    if Path::exists(path) {
        let contents = read_to_string(path).unwrap();

        serde_yaml::from_str(&contents).unwrap()
    }

    ConfigInProgress {
        schedule: None,
        first_execution_date: None,
        first_execution_time: None,
        desired_value: DesiredValue::Message,
        has_token: None,
        chat_id: None,
        message: None,
        token: None,
    }
}

fn save_state(u_id: &String, config_in_progress: ConfigInProgress) {
    let file_path = format!("in_progress/{}", &u_id);
    let path = Path::new(&file_path);
    let contents = serde_yaml::to_string(&config_in_progress).unwrap();
    let mut file = File::create(path).unwrap();
    let _ = file.write_all(contents.as_bytes());
}

fn delete_state(u_id: &String) {
    let file_path = format!("in_progress/{}", &u_id);
    let path = Path::new(&file_path);

    if Path::exists(path) {
        let _ = remove_file(file_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
