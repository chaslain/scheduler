extern crate event_manager;

use chrono::Datelike;
use chrono::Month;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono::Utc;
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
    Cancelled
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
    StartMonth,
    StartDay,
    StartTime,
    Chat,
    HasToken,
    Token,
    None
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

impl UserInput {
    pub fn parse_frequency(frequency: &String) -> Result<crate::UserInput, ()> {
        match frequency.to_lowercase().as_str() {
            "daily" => Ok(UserInput::Frequency(Schedule::Daily)),
            "weekly" => Ok(UserInput::Frequency(Schedule::Weekly)),
            "biweekly" => Ok(UserInput::Frequency(Schedule::Biweekly)),
            "monthly" => Ok(UserInput::Frequency(Schedule::Monthly)),
            "yearly" => Ok(UserInput::Frequency(Schedule::Yearly)),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ConfigInProgress {
    pub schedule: Option<Schedule>,
    pub desired_value: DesiredValue,
    pub first_execution_month: Option<String>,
    pub first_execution_day: Option<String>,
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
            DesiredValue::Frequency => {
                match self.schedule.as_ref().unwrap() {
                    Schedule::Daily => DesiredValue::StartTime,
                    _ => DesiredValue::StartMonth,
                }
            },
            DesiredValue::StartMonth => DesiredValue::StartDay,
            DesiredValue::StartDay => DesiredValue::StartTime,
            DesiredValue::StartTime => DesiredValue::Chat,
            DesiredValue::Chat => DesiredValue::HasToken,
            DesiredValue::HasToken => {
                if self.has_token.unwrap() {
                    DesiredValue::Token
                }
                else {
                    DesiredValue::None
                }
            },
            DesiredValue::Token => DesiredValue::None,
            DesiredValue::None => DesiredValue::None
        }
    }

    pub fn get_flow_status(&self) -> FlowStatus {
        match self.desired_value {
            DesiredValue::Message => FlowStatus::Step(self.get_message()),
            DesiredValue::Frequency => FlowStatus::Step(self.get_message()),
            DesiredValue::StartMonth => FlowStatus::Step(self.get_message()),
            DesiredValue::StartDay => FlowStatus::Step(self.get_message()),
            DesiredValue::StartTime => FlowStatus::Step(self.get_message()),
            DesiredValue::Chat => FlowStatus::Step(self.get_message()),
            DesiredValue::HasToken => FlowStatus::Step(self.get_message()),
            DesiredValue::Token => match self.has_token {
                Some(has_token) => match has_token {
                    true => FlowStatus::Done,
                    false => FlowStatus::Step(self.get_message()),
                },
                None => FlowStatus::Step(self.get_message()),
            },
            DesiredValue::None => FlowStatus::Done
        }
    }

    fn get_message(&self) -> Coorespondance {
        match self.desired_value {
            DesiredValue::Message => Coorespondance {
                message: "Send the message or media you'd like sent.".to_string(),
                option_type: OptionType::Media,
            },
            DesiredValue::Frequency => Coorespondance {
                message: "How often would you like this sent?".to_string(),
                option_type: OptionType::Options(vec![
                    vec![
                    "Daily".to_owned(),
                    "Weekly".to_owned(),
                    "Biweekly".to_owned(),
                    "Monthly".to_owned(),
                    "Yearly".to_owned()
                    ]
                ]),
            },
            DesiredValue::StartMonth => Coorespondance {
                message:
                    "Please select your scheduled month."
                        .to_string(),
                option_type: OptionType::Options(get_option_months()),
            },
            DesiredValue::StartDay => Coorespondance { option_type: OptionType::Options(get_option_days(self.first_execution_month.as_ref().unwrap())), message: "Please select a day of month".to_string() },
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
            DesiredValue::None => Coorespondance { option_type: OptionType::None, message: "Your message is scheduled. Thank you!".to_owned() }
        }
    }
}

pub enum OptionType {
    Options(Vec<Vec<String>>),
    Media,
    Time,
    Date,
    YesNo,
    None
}

pub struct Coorespondance {
    pub option_type: OptionType,
    pub message: String,
}

fn load_input(state: &mut ConfigInProgress, message: &String) -> Result<UserInput, String> {

    if message.to_lowercase() == "/cancel" {
        return Ok(UserInput::Cancel);
    }

    match state.desired_value {
        DesiredValue::Message => Ok(UserInput::Message(message.to_owned())),
        DesiredValue::Chat => Ok(UserInput::Message(message.to_owned())),
        DesiredValue::Frequency => {
            let parse = UserInput::parse_frequency(message);
            if parse.is_err() {
                Err("Please select a frequency.".to_owned())
            } else {
                Ok(parse.unwrap())
            }
        }
        DesiredValue::StartMonth => {

            let parse: Result<i32, ()> = match message.to_lowercase().as_str() {
                "january" => Ok(1),
                "february" => Ok(2),
                "march" => Ok(3),
                "april" => Ok(4),
                "may" => Ok(5),
                "june" => Ok(6),
                "july" => Ok(7),
                "august" => Ok(8),
                "september" => Ok(9),
                "october" => Ok(10),
                "november" => Ok(11),
                "december" => Ok(12),
                _ => Err(())
            };

            match parse {
                Ok(_) => {
                    state.first_execution_month = Some(message.to_owned());
                    Ok(UserInput::Message(message.to_owned()))
                },
                Err(()) => Err("Please use one of the provided buttons to select your month.".to_string())
            }
        },
        DesiredValue::StartDay => {
            let days = get_days_by_month(&state.first_execution_month.as_ref().unwrap());

            match message.parse::<i32>() {
                Ok(number) => {
                    if number < 1 || number > days {
                        Err("Please use one of the provided buttons to select your day.".to_string())
                    }
                    else {
                        Ok(UserInput::Message(message.to_owned()))
                    }
                },
                Err(_) => Err("Please use one of the provided buttons to select your day.".to_string())
            }
        }
        DesiredValue::StartTime => {
            let time = NaiveTime::parse_from_str(message, "%H:%M");

            match time {
                Err(e) => {
                    Err("Hmm... Sorry, I don't understand that. Can you send it in the 24-hour HH:MM format?".to_owned())
                }
                Ok(_) => Ok(UserInput::Time(message.to_owned())),
            }
        }
        DesiredValue::HasToken => Ok(UserInput::YesNo(message.to_lowercase() == "yes")),
        DesiredValue::Token => Ok(UserInput::Message(message.to_owned())),
        DesiredValue::None => Ok(UserInput::Message(message.to_owned()))
    }
}

pub fn accept_incoming_message(u_id: &String, message: &String) -> FlowStatus {
    // first thing we have to do is stick this into an enum.
    let (mut state, already_exists) = get_state(&u_id);

    if !already_exists {
        save_state(u_id, &state);
        state.get_flow_status()
    } else {
        let validate = load_input(&mut state, message);

        match validate {
            Ok(input) => process_incoming_message(u_id, input, &mut state),
            Err(message) => FlowStatus::Error {
                message,
                desired_value: state.desired_value,
            },
        }
    }
}

fn process_incoming_message(
    u_id: &String,
    message: UserInput,
    state: &mut ConfigInProgress,
) -> FlowStatus {
    match message {
        UserInput::Cancel => {
            delete_state(u_id);
            FlowStatus::Cancelled
        }
        _ => {
            let closed = match state.desired_value {
                DesiredValue::Message => process_desired_message(state, message),
                DesiredValue::Frequency => process_frequency(state, message),
                DesiredValue::StartDay => process_month_day(state, message),
                DesiredValue::StartMonth => process_month(state, message),
                DesiredValue::StartTime => process_time(state, message),
                DesiredValue::Chat => process_chat(state, message),
                DesiredValue::HasToken => process_has_token(state, message, u_id),
                DesiredValue::Token => process_token(state, message, u_id),
                DesiredValue::None => true
            };

            if !closed {
                state.move_to_next_step();
                save_state(u_id, state);
                state.get_flow_status()
            } else {
                FlowStatus::Done
            }


        }
    }
}

fn process_desired_message(config_in_progress: &mut ConfigInProgress, message: UserInput) -> bool {
    match message {
        UserInput::Message(desired_message) => {
            config_in_progress.message = Some(Message::Message(desired_message));
        }
        UserInput::Media(id) => config_in_progress.message = Some(Message::Media(id)),
        _ => panic!("Unsupported Input type"),
    }

    false
}

fn process_frequency<'a>(config_in_progress: &mut ConfigInProgress, message: UserInput) -> bool {
    match message {
        UserInput::Frequency(frequency) => config_in_progress.schedule = Some(frequency),
        _ => panic!("Unsupported Input type"),
    }

    false
}

fn process_month_day(config_in_progress: &mut ConfigInProgress, message: UserInput) -> bool {
    match message {
        
        UserInput::Message(date) => {
            config_in_progress.first_execution_day = Some(date);
        }
        _ => panic!("Unsupported Input type"),
    }

    false
}

fn process_month(config_in_progress: &mut ConfigInProgress, message: UserInput) -> bool {
    match message {
        UserInput::Message(date) => config_in_progress.first_execution_month = Some(date),
        _ => panic!("Unsupported Input type"),
    }

    false
}

fn process_time(config_in_progress: &mut ConfigInProgress, message: UserInput) -> bool {
    match message {
        UserInput::Time(time) => config_in_progress.first_execution_time = Some(time),
        _ => panic!("Unsupported Input type"),
    }

    false
}

fn process_chat(confing_in_progress: &mut ConfigInProgress, message: UserInput) -> bool {
    match message {
        UserInput::Message(string) => confing_in_progress.chat_id = Some(string),
        _ => panic!("Unsupported Input Type"),
    }

    false
}

fn process_has_token(config_in_progress: &mut ConfigInProgress, message: UserInput, u_id: &String) -> bool {
    match message {
        UserInput::YesNo(answer) => {
            config_in_progress.has_token = Some(answer);

            if !answer {
                // flow is now done!
                close(u_id, config_in_progress);
                true
            }
            else {
                false
            }
        }

        _ => panic!("Unsupported Input Type"),
    }
}

fn process_token(config_in_progress: &mut ConfigInProgress, message: UserInput, u_id: &String) -> bool {
    match message {
        UserInput::Message(token) => config_in_progress.token = Some(token),
        _ => panic!("Unsupported Input Type"),
    }

    close(u_id, config_in_progress);
    true
}

fn get_first_execution_date(state: &ConfigInProgress) -> NaiveDate {
    let year = Utc::now().year();
    let month = get_month_number(&state.first_execution_month);
    let day = state.first_execution_day.as_ref().unwrap().parse::<u32>().unwrap();

    NaiveDate::from_ymd(year, month, day)
}

fn close(u_id: &String, config_in_progress: &mut ConfigInProgress) {
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
            let date = get_first_execution_date(config_in_progress);
            let schedule = emSchedule::Weekly {
                weekday: date.weekday(),
                time: config_in_progress.first_execution_time.to_owned().unwrap(),
            };

            let _ = create_schedule(u_id, config, schedule);
        }
        Schedule::Biweekly => {
            let date = get_first_execution_date(config_in_progress);

            let schedule = emSchedule::BiWeekly {
                weekday: date.weekday(),
                time: config_in_progress.first_execution_time.to_owned().unwrap(),
                odd: date.iso_week().week() % 2 == 0,
            };

            let _ = create_schedule(u_id, config, schedule);
        }
        Schedule::Monthly => {
            let date = get_first_execution_date(config_in_progress);

            let schedule = emSchedule::Monthly {
                day: date.day() as i32,
                time: config_in_progress.first_execution_time.to_owned().unwrap(),
            };

            let _ = create_schedule(u_id, config, schedule);
        }
        Schedule::Yearly => {
            let date = get_first_execution_date(config_in_progress);

            let schedule = emSchedule::Yearly {
                month: get_month_from_int(date.month() as i32).unwrap(),
                day: date.day() as i32,
                time: config_in_progress.first_execution_time.to_owned().unwrap(),
            };

            let _ = create_schedule(u_id, config, schedule);
        }
    }
}



fn get_state(u_id: &String) -> (ConfigInProgress, bool) {
    let file_name: String = format!("in_progress/{}", u_id);

    let path = Path::new(&file_name);

    if Path::exists(path) {
        let contents = read_to_string(path).unwrap();

        (serde_yaml::from_str(&contents).unwrap(), true)
    } else {
        (
            ConfigInProgress {
                schedule: None,
                first_execution_day: None,
                first_execution_month: None,
                first_execution_time: None,
                desired_value: DesiredValue::Message,
                has_token: None,
                chat_id: None,
                message: None,
                token: None,
            },
            false,
        )
    }
}

fn save_state(u_id: &String, config_in_progress: &ConfigInProgress) {
    let file_path = format!("./in_progress/{}", &u_id);

    let path = Path::new(&file_path);
    let contents = serde_yaml::to_string(config_in_progress).unwrap();
    let file = File::create(path);
    match file {
        Ok(mut f) => {
            let _ = f.write_all(contents.as_bytes());
        }
        Err(err) => {
            panic!("{}", err)
        }
    }
}

fn delete_state(u_id: &String) {
    let file_path = format!("in_progress/{}", &u_id);
    let path = Path::new(&file_path);

    if Path::exists(path) {
        let _ = remove_file(file_path);
    }
}

fn get_days_by_month(month: &String) -> i32 {
    match month.to_lowercase().as_str() {
        "january" => 31,
        "february" => 28,
        "march" => 31,
        "april" => 30,
        "may" => 31,
        "june" => 30,
        "july" => 31,
        "august" => 31,
        "september" => 30,
        "october" => 31,
        "november" => 30,
        "december" => 31,
        _ => 0
    }
}

fn get_month_number(option: &Option<String>) -> u32 {
    match option {
        Some(month) => match month.to_lowercase().as_str() {
            "january" => 1,
            "february" => 2,
            "march" => 3,
            "april" => 4,
            "may" => 5,
            "june" => 6,
            "july" => 7,
            "august" => 8,
            "september" => 9,
            "october" => 10,
            "november" => 11,
            "december" => 12,
            _ => 0
        },
        None => 0
    }
    
}

fn get_option_days(month: &String) -> Vec<Vec<String>> {


    let mut result: Vec<Vec<String>> = Vec::new();

    result.push(Vec::new());
    result.push(Vec::new());
    result.push(Vec::new());
    result.push(Vec::new());
    result.push(Vec::new());

    let mut i = 1;
    let mut j = 0;
    let mut index = 0;
    while i <= get_days_by_month(month) {
        result.get_mut(index).unwrap().push(i.to_string());
        i+=1;
        j+=1;

        if j == 7 {
            j = 0;
            index += 1;
        }
    }

    result
}


fn get_option_months() -> Vec<Vec<String>> {
    vec!(
        vec!("January".to_owned()),
        vec!("February".to_owned()),
        vec!("March".to_owned()),
        vec!("April".to_owned()),
        vec!("May".to_owned()),
        vec!("June".to_owned()),
        vec!("July".to_owned()),
        vec!("August".to_owned()),
        vec!("September".to_owned()),
        vec!("October".to_owned()),
        vec!("November".to_owned()),
        vec!("December".to_owned())
    )
}

fn get_month_from_int(item: i32) -> Option<Month>{
    match item {
        1 => Some(Month::January),
        2 => Some(Month::February),
        3 => Some(Month::March),
        4 => Some(Month::April),
        5 => Some(Month::May),
        6 => Some(Month::June),
        7 => Some(Month::July),
        8 => Some(Month::August),
        9 => Some(Month::September),
        10 => Some(Month::October),
        11 => Some(Month::November),
        12 => Some(Month::December),
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
