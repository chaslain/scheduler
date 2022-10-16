use chrono::{Month, Weekday};
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, read_link, read_to_string, remove_file, File},
    io::Write,
    os::unix::fs::symlink,
    path::Path,
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub chat_id: String,
    pub message: Message,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Message {
    Message(String),
    Photo(String),
    Video(String),
    Audio(String),
    Document(String),
}

pub enum Schedule {
    Daily {
        time: String,
    },
    Weekly {
        weekday: Weekday,
        time: String,
    },
    BiWeekly {
        weekday: Weekday,
        time: String,
        odd: bool,
    },
    Monthly {
        day: i32,
        time: String,
    },
    Yearly {
        day: i32,
        time: String,
        month: Month,
    },
}

fn get_weekday_display(weekday: Weekday) -> String {
    match weekday {
        Weekday::Sun => String::from("sunday"),
        Weekday::Mon => String::from("monday"),
        Weekday::Tue => String::from("tuesday"),
        Weekday::Wed => String::from("wednesday"),
        Weekday::Thu => String::from("thursday"),
        Weekday::Fri => String::from("friday"),
        Weekday::Sat => String::from("saturday"),
    }
}

impl Schedule {
    pub fn get_file_location(self, u_id: &String) -> String {
        match self {
            Schedule::Daily { time } => {
                format!("recurring/daily/{}/{}", &time, &u_id)
            }

            Schedule::Weekly { weekday, time } => {
                format!(
                    "recurring/weekly/{}/{}/{}",
                    &get_weekday_display(weekday),
                    &time,
                    &u_id
                )
            }

            Schedule::BiWeekly { weekday, time, odd } => {
                let odd_string = if odd { "1" } else { "0" };
                format!(
                    "recurring/biweekly/{}/{}/{}/{}",
                    &odd_string,
                    &get_weekday_display(weekday),
                    &time,
                    &u_id
                )
            }

            Schedule::Monthly { day, time } => {
                format!("recurring/monthly/{}/{}/{}", &day, &time, &u_id)
            }

            Schedule::Yearly { day, time, month } => {
                format!(
                    "recurring/yearly/{}/{}/{}/{}",
                    &month.number_from_month(),
                    &day,
                    &time,
                    &u_id
                )
            }
        }
    }
}

pub fn create_schedule(user_id: &String, configuration: Config, schedule: Schedule) -> bool {
    match create_schedule_main(user_id, configuration, schedule) {
        Ok(path) => create_schedule_index(user_id, &path),
        Err(()) => false,
    }
}

fn create_schedule_index(user_id: &String, path_str: &String) -> bool {
    let path = Path::new(path_str);
    let sym_directory = format!("users/{}", user_id);
    let sym_path_directory = Path::new(&sym_directory);

    let mut i = 1;

    if sym_path_directory.exists() {
        for _ in sym_path_directory.read_dir().unwrap() {
            i += 1;
        }
    } else {
        _ = create_dir_all(sym_path_directory);
    }

    let sym = format!("users/{}/{}", user_id, i.to_string());
    let sym_path = Path::new(&sym);

    match symlink(path, sym_path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn create_schedule_main(
    user_id: &String,
    configuration: Config,
    schedule: Schedule,
) -> Result<String, ()> {
    let file_content = serde_yaml::to_string(&configuration).unwrap();

    let file_name = schedule.get_file_location(user_id);

    let path = Path::new(&file_name);

    create_dir_all(&path.parent().unwrap()).unwrap();

    let mut file = File::create(path).unwrap();

    match file.write_all(file_content.as_bytes()) {
        Ok(_) => Ok(file_name),
        Err(_) => Err(()),
    }
}

pub fn delete_scheduled(user_id: &String, number: i32) {
    let sym_file_path = format!("users/{}/{}", user_id, number);

    let sym_path = Path::new(&sym_file_path);

    if sym_path.exists() {
        match read_link(sym_path) {
            Ok(path) => {
                _ = remove_file(path);
            }
            Err(_) => {}
        }

        _ = remove_file(sym_path);
    }
}

pub fn get_config(user_id: &String, number: i32) -> Result<Config, ()> {
    let sym_file_path = format!("users/{}/{}", user_id, number);

    let sym_path = Path::new(&sym_file_path);

    match read_link(sym_path) {
        Ok(file) => {
            let data = read_to_string(file).unwrap();
            Ok(serde_yaml::from_str(&data).unwrap())
        }
        Err(_) => Err(()),
    }
}

// pub fn list_user_jobs(user_id: &String) -> Vec<Config> {

// }

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_daily() {
        let schedule: Schedule = Schedule::Daily {
            time: "12:02".to_owned(),
        };
        let res = schedule.get_file_location("test".as_ref());

        assert_eq!(res, "recurring/daily/12:02/test")
    }

    #[test]
    pub fn test_weekly() {
        let schedule: Schedule = Schedule::Weekly {
            weekday: Weekday::Mon,
            time: "3:02".to_owned(),
        };
        let res = schedule.get_file_location("test2".as_ref());

        assert_eq!(res, "recurring/weekly/monday/3:02/test2")
    }

    #[test]
    pub fn test_monthly() {
        let schedule: Schedule = Schedule::Monthly {
            day: 3,
            time: "10:56".to_owned(),
        };
        let res = schedule.get_file_location("alan".as_ref());

        assert_eq!(res, "recurring/monthly/3/10:56/alan")
    }

    #[test]
    pub fn test_biweekly() {
        let schedule: Schedule = Schedule::BiWeekly {
            weekday: Weekday::Tue,
            time: "00:00".to_owned(),
            odd: true,
        };
        let res = schedule.get_file_location("bob".as_ref());

        assert_eq!("recurring/biweekly/1/tuesday/00:00/bob", res)
    }

    #[test]
    pub fn test_yearly() {
        let schedule: Schedule = Schedule::Yearly {
            day: 25,
            time: "10:00".to_owned(),
            month: Month::December,
        };
        let res: String = schedule.get_file_location("santa".as_ref());

        assert_eq!("recurring/yearly/12/25/10:00/santa", res)
    }
}
