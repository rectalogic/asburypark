use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeSet, fmt::Display, ops::Range};

mod object;
pub use object::restaurants_value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Restaurants(Vec<Restaurant>);

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Restaurant {
    name: String,
    url: String,
    map_id: String, // Use with https://maps.app.goo.gl/{map_id}
    instagram_id: String,
    verified: NaiveDate,
    kind: Kind,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Kind {
    Byob,
    Other,
    Closed,
    HappyHour {
        description: Vec<String>,
        menu_url: Option<String>,
        happytimes: HappyTimes,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct HappyTimes(Vec<DayHours>);

#[derive(Serialize, Deserialize, Clone, Debug)]
enum DayHours {
    Single(Day, Hours),
    Range((Day, Day), Hours),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Day {
    Sun = 0,
    Mon = 1,
    Tue = 2,
    Wed = 3,
    Thu = 4,
    Fri = 5,
    Sat = 6,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
struct Hours(Hour, Hour);

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Hour(#[serde(deserialize_with = "deserialize_hour")] u16);

struct HumanTime {
    description: String,
    data_attributes: String,
}

impl HappyTimes {
    fn as_data_attributes(&self) -> String {
        let dayhour_set = self
            .0
            .iter()
            .flat_map(|dh| dh.as_tuples())
            .collect::<BTreeSet<_>>();

        format!(
            r#"data-daytimes="{}""#,
            Self::data_daytimes(dayhour_set.iter().copied())
        )
    }

    fn as_human_readable(&self) -> Vec<HumanTime> {
        self.0
            .iter()
            .map(|dh| HumanTime {
                description: format!("{}", dh),
                data_attributes: dh.as_data_attributes(),
            })
            .collect::<Vec<_>>()
    }

    fn data_daytimes(dayhour_tuples: impl Iterator<Item = (Day, u16)>) -> String {
        let dayhour_tuples: Vec<_> = dayhour_tuples.collect();
        let days = dayhour_tuples
            .iter()
            .map(|(d, _)| d)
            .collect::<BTreeSet<_>>();
        let hours = dayhour_tuples
            .iter()
            .map(|(_, h)| h)
            .collect::<BTreeSet<_>>();
        dayhour_tuples
            .iter()
            .map(|(d, h)| format!("{}-{h}", *d as isize))
            .chain(days.into_iter().map(|d| format!("{}-all", *d as isize)))
            .chain(hours.into_iter().map(|h| format!("all-{h}")))
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
    let h = Hour(u16::deserialize(deserializer)?);
    let ch = h.wraparound();
    let hours = h.hours();
    let minutes = ch.minutes();

    // Validate minutes and hours
    if hours > Hours::END_HOUR || minutes > 59 {
        return Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Unsigned(h.0 as u64),
            &format!(
                "an hour/minutes between {} and {} with minutes between 0 and 59",
                Hours::START_HOUR * 100,
                Hours::END_HOUR * 100
            )
            .as_str(),
        ));
    }
    Ok(h.0)
}

impl DayHours {
    fn as_tuples(&self) -> impl Iterator<Item = (Day, u16)> {
        let (days, hours) = match self {
            DayHours::Single(day, hours) => ((*day, *day), hours),
            DayHours::Range(days, hours) => (*days, hours),
        };
        iter_days(days).flat_map(|day| hours.as_range().map(move |h| (day, h)))
    }

    fn as_data_attributes(&self) -> String {
        format!(
            r#"data-daytimes="{}""#,
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
    fn iter() -> std::array::IntoIter<Day, 7> {
        [
            Day::Sun,
            Day::Mon,
            Day::Tue,
            Day::Wed,
            Day::Thu,
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
            Self::Tue => write!(f, "Tue"),
            Self::Wed => write!(f, "Wed"),
            Self::Thu => write!(f, "Thu"),
            Self::Fri => write!(f, "Fri"),
            Self::Sat => write!(f, "Sat"),
        }
    }
}

impl Hour {
    fn wraparound(&self) -> Self {
        // Allow 2400–2700 by wrapping into the 0–300 range
        if (2400..=Hours::END_HOUR * 100).contains(&self.0) {
            Self(self.0 - 2400)
        } else {
            Self(self.0)
        }
    }

    fn hours(&self) -> u16 {
        self.0 / 100
    }
    fn minutes(&self) -> u16 {
        self.0 % 100
    }
}

impl Display for Hour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = self.wraparound();

        let hours = t.0 / 100;
        let minutes = t.0 % 100;

        // AM/PM conversion
        let (display_hour, suffix) = match hours {
            0 => (12, "am"),
            1..=11 => (hours, "am"),
            12 => (12, "pm"),
            13..=23 => (hours - 12, "pm"),
            _ => unreachable!(),
        };

        if minutes == 0 {
            write!(f, "{display_hour}{suffix}")
        } else {
            write!(f, "{display_hour}:{minutes:02}{suffix}")
        }
    }
}

impl Hours {
    const START_HOUR: u16 = 9;
    const END_HOUR: u16 = 25;

    fn as_range(&self) -> Range<u16> {
        // Truncate to nearest hour, e.g. 1630 -> 16
        let mut start = self.0.hours();
        let mut end = self.1.hours();
        // If start has minutes, bump to next hour
        if self.0.minutes() > 0 {
            start += 1;
        }
        // If end has minutes, bump to next hour
        if self.1.minutes() > 0 {
            end += 1;
        }
        start..end
    }
}

impl Display for Hours {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use crate::ron_options;

    use super::*;
    use ron::ser::PrettyConfig;

    #[test]
    fn test_happytimes_dataattr() {
        assert_eq!(
            HappyTimes(vec![DayHours::Single(
                Day::Wed,
                Hours(Hour(1300), Hour(1500))
            )])
            .as_data_attributes(),
            r#"data-daytimes="3-13 3-14 3-all all-13 all-14""#
        );
        assert_eq!(
            HappyTimes(vec![
                DayHours::Single(Day::Wed, Hours(Hour(1300), Hour(1500))),
                DayHours::Range((Day::Wed, Day::Fri), Hours(Hour(1400), Hour(1600))),
            ])
            .as_data_attributes(),
            r#"data-daytimes="3-13 3-14 3-15 4-14 4-15 5-14 5-15 3-all 4-all 5-all all-13 all-14 all-15""#
        );
        assert_eq!(
            HappyTimes(vec![DayHours::Single(
                Day::Mon,
                Hours(Hour(1330), Hour(1530))
            )])
            .as_data_attributes(),
            r#"data-daytimes="1-14 1-15 1-all all-14 all-15""#
        );
        assert_eq!(
            HappyTimes(vec![DayHours::Range(
                (Day::Sat, Day::Sun),
                Hours(Hour(1000), Hour(1200))
            )])
            .as_data_attributes(),
            r#"data-daytimes="0-10 0-11 6-10 6-11 0-all 6-all all-10 all-11""#
        );
    }

    #[test]
    fn test_dayhours_dataattr() {
        assert_eq!(
            DayHours::Single(Day::Wed, Hours(Hour(1300), Hour(1500))).as_data_attributes(),
            r#"data-daytimes="3-13 3-14 3-all all-13 all-14""#
        );
        assert_eq!(
            DayHours::Range((Day::Wed, Day::Fri), Hours(Hour(1300), Hour(1700)))
                .as_data_attributes(),
            r#"data-daytimes="3-13 3-14 3-15 3-16 4-13 4-14 4-15 4-16 5-13 5-14 5-15 5-16 3-all 4-all 5-all all-13 all-14 all-15 all-16""#
        );
    }

    #[test]
    fn test_dayrange() {
        assert_eq!(
            iter_days((Day::Mon, Day::Thu)).collect::<Vec<_>>(),
            vec![Day::Mon, Day::Tue, Day::Wed, Day::Thu]
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
        assert_eq!("9:30am-2pm", format!("{}", Hours(Hour(930), Hour(1400))));
        assert_eq!("11pm-1am", format!("{}", Hours(Hour(2300), Hour(2500))));
    }

    #[test]
    fn test_hours_deserialize() {
        let ro = ron_options();
        assert_eq!(
            ro.from_str("(2400, 2500)"),
            Ok(Hours(Hour(2400), Hour(2500)))
        );
        assert_eq!(
            ro.from_str("(2100, 2500)"),
            Ok(Hours(Hour(2100), Hour(2500)))
        );
        assert_eq!(
            ro.from_str("(2300, 2500)"),
            Ok(Hours(Hour(2300), Hour(2500)))
        );
        assert!(ro.from_str::<Hours>("(1300, 2900)").is_err());
        assert!(ro.from_str::<Hours>("(2399, 2500)").is_err());
        assert!(ro.from_str::<Hours>("(2300, 9900)").is_err());
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
                    DayHours::Single(Day::Mon, Hours(Hour(1600), Hour(1800))),
                    DayHours::Single(Day::Tue, Hours(Hour(1600), Hour(2200))),
                    DayHours::Range((Day::Wed, Day::Fri), Hours(Hour(1600), Hour(1800))),
                ]),
            },
        }]);
        let pretty = PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let s = ron_options()
            .to_string_pretty(&restaurants, pretty)
            .expect("Serialization failed");

        println!("{}", s);
    }
}
