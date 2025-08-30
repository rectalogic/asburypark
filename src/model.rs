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
        daytimes: Vec<DayTime>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DayTime {
    Single(Day, Time),
    Range((Day, Day), Time),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Day {
    Sun = 0,
    Mon = 1,
    Tues = 2,
    Wed = 3,
    Thurs = 4,
    Fri = 5,
    Sat = 6,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Time(u16, u16);

#[cfg(test)]
mod tests {
    use super::*;
    use ron::ser::{PrettyConfig, to_string_pretty};

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
                daytimes: vec![
                    DayTime::Single(Day::Mon, Time(1600, 1800)),
                    DayTime::Single(Day::Tues, Time(1600, 2200)),
                    DayTime::Range((Day::Wed, Day::Fri), Time(1600, 1800)),
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
