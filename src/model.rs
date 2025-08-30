use std::fmt::Display;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Restaurants(Vec<Restaurant>);

#[derive(Serialize, Deserialize, Debug)]
pub struct Restaurant {
    name: String,
    url: String,
    map_id: String, // Use with https://maps.app.goo.gl/{map_id}
    instagram_id: String,
    verified: NaiveDate,
    kind: Kind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Kind {
    Byob,
    Other,
    Closed,
    HappyHour {
        description: Vec<String>,
        menu_url: Option<String>,
        dayhours: Vec<DayHours>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DayHours {
    Single(Day, Hours),
    Range((Day, Day), Hours),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Day {
    Sun = 0,
    Mon = 1,
    Tues = 2,
    Wed = 3,
    Thurs = 4,
    Fri = 5,
    Sat = 6,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Hours(u16, u16);

impl DayHours {
    pub fn css_data_days(&self) -> String {
        match self {
            Self::Single(day, _) => format!("{}", *day as i32),
            Self::Range(days, _) => DayVec::try_from(days)
                .expect("Invalid days {days:?}")
                .0
                .iter()
                .map(|d| (*d as i32).to_string())
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    pub fn css_data_hours(&self) -> String {
        match self {
            Self::Single(_, hours) | Self::Range(_, hours) => {
                // Truncate to nearest hour, e.g. 1630 -> 16
                let start = hours.0 / 100;
                let end = hours.1 / 100;
                (start..end)
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }
    }

    pub fn human_daytimes(&self) -> String {
        match self {
            Self::Single(day, hours) => format!("{day} {hours}"),
            Self::Range((startday, endday), hours) => format!("{startday}-{endday} {hours}"),
        }
    }
}

#[derive(Debug, PartialEq)]
struct DayVec(Vec<Day>);

impl TryFrom<&(Day, Day)> for DayVec {
    type Error = &'static str;

    fn try_from(days: &(Day, Day)) -> Result<Self, Self::Error> {
        if days.0 > days.1 {
            Err("Start day must be before end day")
        } else if days.0 == days.1 {
            Ok(DayVec(vec![days.0]))
        } else {
            let results: Result<Vec<_>, _> = ((days.0 as i32)..=(days.1 as i32))
                .map(Day::try_from)
                .collect();
            Ok(DayVec(results?))
        }
    }
}

impl TryFrom<i32> for Day {
    type Error = &'static str;

    fn try_from(num: i32) -> Result<Self, Self::Error> {
        match num {
            0 => Ok(Day::Sun),
            1 => Ok(Day::Mon),
            2 => Ok(Day::Tues),
            3 => Ok(Day::Wed),
            4 => Ok(Day::Thurs),
            5 => Ok(Day::Fri),
            6 => Ok(Day::Sat),
            _ => Err("Invalid day number {num}"),
        }
    }
}

impl Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sun => write!(f, "Sun"),
            Self::Mon => write!(f, "Mon"),
            Self::Tues => write!(f, "Tues"),
            Self::Wed => write!(f, "Wed"),
            Self::Thurs => write!(f, "Thurs"),
            Self::Fri => write!(f, "Fri"),
            Self::Sat => write!(f, "Sat"),
        }
    }
}

impl Display for Hours {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!() //XXX convert e.g. 1300 to 1:30pm
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ron::ser::{PrettyConfig, to_string_pretty};

    #[test]
    fn test_dayrange() {
        assert_eq!(
            DayVec::try_from(&(Day::Mon, Day::Thurs)),
            Ok(DayVec(vec![Day::Mon, Day::Tues, Day::Wed, Day::Thurs]))
        );
    }

    #[test]
    fn test_save() {
        let restaurants = Restaurants(vec![Restaurant {
            name: "The Black Swan".into(),
            url: "https://www.theblackswanap.com/".into(),
            map_id: "JiKYhYvKsK2ysBZs9".into(),
            instagram_id: "theblackswanap".into(),
            verified: NaiveDate::from_ymd_opt(2025, 7, 28).unwrap(),
            kind: Kind::HappyHour {
                description: vec![
                    "50% off all alcohol, HH food menu".into(),
                    "Wed 2nd burger $5".into(),
                ],
                menu_url: Some("https://www.theblackswanap.com/happy-hour".into()),
                dayhours: vec![
                    DayHours::Single(Day::Mon, Hours(1600, 1800)),
                    DayHours::Single(Day::Tues, Hours(1600, 2200)),
                    DayHours::Range((Day::Wed, Day::Fri), Hours(1600, 1800)),
                ],
            },
        }]);
        let pretty = PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let s = to_string_pretty(&restaurants, pretty).expect("Serialization failed");

        println!("{}", s);
    }
}
