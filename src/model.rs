use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeSet, fmt::Display, ops::Range};

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
        happytimes: HappyTimes,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HappyTimes(Vec<DayHours>);

#[derive(Serialize, Deserialize, Debug)]
pub enum DayHours {
    Single(Day, Hours),
    Range((Day, Day), Hours),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Day {
    Sun = 0,
    Mon = 1,
    Tues = 2,
    Wed = 3,
    Thurs = 4,
    Fri = 5,
    Sat = 6,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Hours(
    #[serde(deserialize_with = "deserialize_hour")] u16,
    #[serde(deserialize_with = "deserialize_hour")] u16,
);

pub struct HumanTime {
    pub description: String,
    pub data_attributes: String,
}

impl HappyTimes {
    pub fn as_data_attributes(&self) -> String {
        let dayhour_set = self
            .0
            .iter()
            .flat_map(|dh| dh.as_tuples())
            .collect::<BTreeSet<_>>();

        format!(
            r#"data-days="{}" data-hours="{}" data-daytimes="{}""#,
            Self::data_days(dayhour_set.iter().copied()),
            Self::data_hours(dayhour_set.iter().copied()),
            Self::data_daytimes(dayhour_set.iter().copied())
        )
    }

    pub fn as_human_readable(&self) -> Vec<HumanTime> {
        self.0
            .iter()
            .map(|dh| HumanTime {
                description: format!("{}", dh),
                data_attributes: dh.as_data_attributes(),
            })
            .collect::<Vec<_>>()
    }

    fn data_days(dayhour_tuples: impl Iterator<Item = (Day, u16)>) -> String {
        dayhour_tuples
            .map(|(day, _)| day)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|d| (d as isize).to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn data_hours(dayhour_tuples: impl Iterator<Item = (Day, u16)>) -> String {
        dayhour_tuples
            .map(|(_, hour)| hour)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn data_daytimes(dayhour_tuples: impl Iterator<Item = (Day, u16)>) -> String {
        dayhour_tuples
            .map(|(d, h)| format!("{}-{h}", d as isize))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn iter_days(days: (Day, Day)) -> impl Iterator<Item = Day> {
    let start = days.0 as usize;
    let end = days.1 as usize;

    // length of inclusive span going forward, wrapping mod 7
    let len = (end + 7 - start) % 7 + 1;

    Day::iter().cycle().skip(start).take(len)
}

fn deserialize_hour<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let h = u16::deserialize(deserializer)?;
    let ch = wraparound_hour(h);
    let hours = ch / 100;
    let minutes = ch % 100;

    // Validate minutes and hours
    if hours > Hours::END_HOUR || minutes > 59 {
        return Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(h as u64),
            &"an hour between 0 and 23 and minute between 0 and 59",
        ));
    }
    Ok(h)
}

impl DayHours {
    fn as_tuples(&self) -> impl Iterator<Item = (Day, u16)> {
        let (days, hours) = match self {
            DayHours::Single(day, hours) => ((*day, *day), hours),
            DayHours::Range(days, hours) => (*days, hours),
        };
        iter_days(days).flat_map(|day| hours.as_range().map(move |h| (day, h)))
    }

    pub fn as_data_attributes(&self) -> String {
        format!(
            r#"data-days="{}" data-hours="{}" data-daytimes="{}""#,
            HappyTimes::data_days(self.as_tuples()),
            HappyTimes::data_hours(self.as_tuples()),
            HappyTimes::data_daytimes(self.as_tuples()),
        )
    }
}

impl Display for DayHours {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(day, hours) => write!(f, "{day} {hours}"),
            Self::Range((startday, endday), hours) => write!(f, "{startday}-{endday} {hours}"),
        }
    }
}

impl Day {
    pub fn iter() -> std::array::IntoIter<Day, 7> {
        [
            Day::Sun,
            Day::Mon,
            Day::Tues,
            Day::Wed,
            Day::Thurs,
            Day::Fri,
            Day::Sat,
        ]
        .into_iter()
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

fn wraparound_hour(t: u16) -> u16 {
    // Allow 2400–2500 by wrapping into the 0–300 range
    if (2400..=Hours::END_HOUR * 100).contains(&t) {
        t - 2400
    } else {
        t
    }
}

fn format_hour(mut t: u16) -> String {
    t = wraparound_hour(t);

    let hours = t / 100;
    let minutes = t % 100;

    // AM/PM conversion
    let (display_hour, suffix) = match hours {
        0 => (12, "am"),
        1..=11 => (hours, "am"),
        12 => (12, "pm"),
        13..=23 => (hours - 12, "pm"),
        _ => unreachable!(),
    };

    if minutes == 0 {
        format!("{display_hour}{suffix}")
    } else {
        format!("{display_hour}:{minutes:02}{suffix}")
    }
}

impl Hours {
    pub const START_HOUR: u16 = 9;
    pub const END_HOUR: u16 = 25;

    fn as_range(&self) -> Range<u16> {
        // Truncate to nearest hour, e.g. 1630 -> 16
        let mut start = self.0 / 100;
        let mut end = self.1 / 100;
        // If start has minutes, bump to next hour
        if self.0 % 100 > 0 {
            start += 1;
        }
        // If end has minutes, bump to next hour
        if self.1 % 100 > 0 {
            end += 1;
        }
        start..end
    }
}

impl Display for Hours {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", format_hour(self.0), format_hour(self.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ron::{
        de::from_str,
        ser::{PrettyConfig, to_string_pretty},
    };

    #[test]
    fn test_happytimes_dataattr() {
        assert_eq!(
            HappyTimes(vec![DayHours::Single(Day::Wed, Hours(1300, 1500))]).as_data_attributes(),
            r#"data-days="3" data-hours="13 14" data-daytimes="3-13 3-14""#
        );
        assert_eq!(
            HappyTimes(vec![
                DayHours::Single(Day::Wed, Hours(1300, 1500)),
                DayHours::Range((Day::Wed, Day::Fri), Hours(1400, 1600)),
            ])
            .as_data_attributes(),
            r#"data-days="3 4 5" data-hours="13 14 15" data-daytimes="3-13 3-14 3-15 4-14 4-15 5-14 5-15""#
        );
        assert_eq!(
            HappyTimes(vec![DayHours::Single(Day::Mon, Hours(1330, 1530))]).as_data_attributes(),
            r#"data-days="1" data-hours="14 15" data-daytimes="1-14 1-15""#
        );
        assert_eq!(
            HappyTimes(vec![DayHours::Range(
                (Day::Sat, Day::Sun),
                Hours(1000, 1200)
            )])
            .as_data_attributes(),
            r#"data-days="0 6" data-hours="10 11" data-daytimes="0-10 0-11 6-10 6-11""#
        );
    }

    #[test]
    fn test_dayhours_dataattr() {
        assert_eq!(
            DayHours::Single(Day::Wed, Hours(1300, 1500)).as_data_attributes(),
            r#"data-days="3" data-hours="13 14" data-daytimes="3-13 3-14""#
        );
        assert_eq!(
            DayHours::Range((Day::Wed, Day::Fri), Hours(1300, 1700)).as_data_attributes(),
            r#"data-days="3 4 5" data-hours="13 14 15 16" data-daytimes="3-13 3-14 3-15 3-16 4-13 4-14 4-15 4-16 5-13 5-14 5-15 5-16""#
        );
    }

    #[test]
    fn test_dayrange() {
        assert_eq!(
            iter_days((Day::Mon, Day::Thurs)).collect::<Vec<_>>(),
            vec![Day::Mon, Day::Tues, Day::Wed, Day::Thurs]
        );
        assert_eq!(
            iter_days((Day::Sat, Day::Sun)).collect::<Vec<_>>(),
            vec![Day::Sat, Day::Sun]
        );
        assert_eq!(
            iter_days((Day::Fri, Day::Sun)).collect::<Vec<_>>(),
            vec![Day::Fri, Day::Sat, Day::Sun]
        );
        assert_eq!(
            iter_days((Day::Mon, Day::Mon)).collect::<Vec<_>>(),
            vec![Day::Mon]
        );
    }

    #[test]
    fn test_hours_display() {
        assert_eq!("9:30am-2pm", format!("{}", Hours(930, 1400)));
        assert_eq!("11pm-1am", format!("{}", Hours(2300, 2500)));
    }

    #[test]
    fn test_hours_deserialize() {
        assert_eq!(from_str("(2400, 2500)"), Ok(Hours(2400, 2500)));
        assert_eq!(from_str("(2100, 2500)"), Ok(Hours(2100, 2500)));
        assert!(from_str::<Hours>("(1300, 2900)").is_err());
        assert!(from_str::<Hours>("(2399, 2500)").is_err());
        assert!(from_str::<Hours>("(2300, 9900)").is_err());
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
                happytimes: HappyTimes(vec![
                    DayHours::Single(Day::Mon, Hours(1600, 1800)),
                    DayHours::Single(Day::Tues, Hours(1600, 2200)),
                    DayHours::Range((Day::Wed, Day::Fri), Hours(1600, 1800)),
                ]),
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
