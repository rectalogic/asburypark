use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::{BTreeSet, HashSet},
    fmt::Display,
    ops::Range,
};
use tera::Function;

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

impl Display for DayHours {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(day, hours) => write!(f, "{day} {hours}"),
            Self::Range((startday, endday), hours) => write!(f, "{startday}-{endday} {hours}"),
        }
    }
}

impl HappyTimes {
    pub fn to_css_data_attributes(&self) -> String {
        let data_days = self
            .0
            .iter()
            .flat_map(|dh| match dh {
                DayHours::Single(day, _) => vec![*day],
                DayHours::Range(days, _) => DayVec::from(days).0.into_iter().collect::<Vec<_>>(),
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|d| (d as isize).to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let data_hours = self
            .0
            .iter()
            .flat_map(|dh| match dh {
                DayHours::Single(_, hours) | DayHours::Range(_, hours) => hours.to_range(),
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        fn dayhours(day: &Day, hours: &Hours) -> impl Iterator<Item = (isize, u16)> {
            let day = *day as isize;
            hours.to_range().map(move |h| (day, h))
        }

        let data_daytimes = self
            .0
            .iter()
            .flat_map(|dh| match dh {
                DayHours::Single(day, hours) => dayhours(day, hours).collect::<Vec<_>>(),
                DayHours::Range(days, hours) => DayVec::from(days)
                    .0
                    .iter()
                    .flat_map(|day| dayhours(day, hours))
                    .collect::<Vec<_>>(),
            })
            .collect::<BTreeSet<(isize, u16)>>()
            .into_iter()
            .map(|(d, h)| format!("{d}-{h}"))
            .collect::<Vec<_>>()
            .join(" ");

        format!(
            r#"data-days="{data_days}" data-hours="{data_hours}" data-daytimes="{data_daytimes}""#
        )
    }

    pub fn human_daytimes(&self) -> String {
        self.0
            .iter()
            .map(|dh| format!("{}", dh))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug, PartialEq)]
struct DayVec(Vec<Day>);

impl From<&Day> for DayVec {
    fn from(day: &Day) -> Self {
        Self(vec![*day])
    }
}

impl From<&(Day, Day)> for DayVec {
    fn from(days: &(Day, Day)) -> Self {
        // Return days.0->Sun and Sat->days.1
        if days.0 > days.1 {
            Self(
                Day::iter()
                    .skip(days.0 as usize)
                    .chain(Day::iter().take_while(|d| *d <= days.1))
                    .collect(),
            )
        } else if days.0 == days.1 {
            Self(vec![days.0])
        } else {
            Self(
                Day::iter()
                    .skip(days.0 as usize)
                    .take_while(|d| *d <= days.1)
                    .collect(),
            )
        }
    }
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
    if hours > 23 || minutes > 59 {
        return Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(h as u64),
            &"an hour between 0 and 23 and minute between 0 and 59",
        ));
    }
    Ok(h)
}

impl Day {
    pub fn iter() -> impl Iterator<Item = Day> {
        [
            Day::Sun,
            Day::Mon,
            Day::Tues,
            Day::Wed,
            Day::Thurs,
            Day::Fri,
            Day::Sat,
        ]
        .iter()
        .copied()
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
    // Allow 2400–2700 by wrapping into the 0–300 range
    if (2400..=2700).contains(&t) {
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

    format!("{display_hour}:{minutes:02}{suffix}")
}

impl Hours {
    fn to_range(&self) -> Range<u16> {
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
    fn test_happytimes_css() {
        assert_eq!(
            HappyTimes(vec![DayHours::Single(Day::Wed, Hours(1300, 1500))])
                .to_css_data_attributes(),
            r#"data-days="3" data-hours="13 14" data-daytimes="3-13 3-14""#
        );
        assert_eq!(
            HappyTimes(vec![
                DayHours::Single(Day::Wed, Hours(1300, 1500)),
                DayHours::Range((Day::Wed, Day::Fri), Hours(1400, 1600)),
            ])
            .to_css_data_attributes(),
            r#"data-days="3 4 5" data-hours="13 14 15" data-daytimes="3-13 3-14 3-15 4-14 4-15 5-14 5-15""#
        );
        assert_eq!(
            HappyTimes(vec![DayHours::Single(Day::Mon, Hours(1330, 1530)),])
                .to_css_data_attributes(),
            r#"data-days="1" data-hours="14 15" data-daytimes="1-14 1-15""#
        );
    }

    #[test]
    fn test_dayrange() {
        assert_eq!(
            DayVec::from(&(Day::Mon, Day::Thurs)),
            DayVec(vec![Day::Mon, Day::Tues, Day::Wed, Day::Thurs])
        );
        assert_eq!(
            DayVec::from(&(Day::Sat, Day::Sun)),
            DayVec(vec![Day::Sat, Day::Sun])
        );
        assert_eq!(
            DayVec::from(&(Day::Fri, Day::Sun)),
            DayVec(vec![Day::Fri, Day::Sat, Day::Sun])
        );
        assert_eq!(DayVec::from(&(Day::Mon, Day::Mon)), DayVec(vec![Day::Mon]));
    }

    #[test]
    fn test_hours_display() {
        assert_eq!("9:00am-2:00pm", format!("{}", Hours(900, 1400)));
        assert_eq!("1:00am-3:00am", format!("{}", Hours(2500, 2700)));
        assert_eq!("11:00pm-1:00am", format!("{}", Hours(2300, 2500)));
    }

    #[test]
    fn test_hours_deserialize() {
        assert_eq!(from_str("(2500, 2700)"), Ok(Hours(2500, 2700)));
        assert_eq!(from_str("(2300, 2500)"), Ok(Hours(2300, 2500)));
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
