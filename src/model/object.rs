use std::{iter::once, sync::Arc};

use minijinja::{
    Value, context,
    value::{Enumerator, Object},
};

pub fn restaurants_value(mut restaurants: super::Restaurants) -> Value {
    let mut happy_hour = Vec::new();
    let mut byob = Vec::new();
    let mut other = Vec::new();
    let mut closed = Vec::new();
    let mut min_hour = super::Hour(2500);
    let mut max_hour = super::Hour(0000);
    for restaurant in restaurants.0.drain(..) {
        match restaurant.kind {
            super::Kind::HappyHour { ref happytimes, .. } => {
                happytimes.0.iter().for_each(|dh| match dh {
                    super::DayHours::Single(_, hours) | super::DayHours::Range(_, hours) => {
                        if hours.0 < min_hour {
                            min_hour = hours.0;
                        }
                        if hours.1 > max_hour {
                            max_hour = hours.1;
                        }
                    }
                });
                happy_hour.push(restaurant);
            }
            super::Kind::Byob => byob.push(restaurant),
            super::Kind::Other => other.push(restaurant),
            super::Kind::Closed => closed.push(restaurant),
        }
    }

    let hour_range = if max_hour.minutes() > 0 {
        min_hour.hours()..(max_hour.hours() + 1)
    } else {
        min_hour.hours()..max_hour.hours()
    };

    context! {
        happy_hour => Value::make_iterable(move || happy_hour.clone().into_iter().map(Value::from_object)),
        byob => Value::make_iterable(move || byob.clone().into_iter().map(Value::from_object)),
        other => Value::make_iterable(move || other.clone().into_iter().map(Value::from_object)),
        closed => Value::make_iterable(move || closed.clone().into_iter().map(Value::from_object)),
        hour_options => Value::from_serialize(
            hour_range.clone()
                .map(|h| {
                    let display = match h {
                        12 => "12pm".to_string(),
                        1..12 => format!("{h}am"),
                        25.. => format!("{}am", h - 24),
                        13..=24 => format!("{}pm", h - 12),
                        _ => unreachable!(),
                    };
                    (h, display)
                })
                .collect::<Vec<_>>(),
        ),
        day_options => Value::from_serialize(
            super::Day::iter()
                .map(|d| (d as isize, format!("{d}")))
                .collect::<Vec<_>>(),
        ),
        dayhours => Value::from(
            (0..=6)
                .map(|d| d.to_string())
                .chain(once("all".to_string()))
                .flat_map(|d| {
                    hour_range.clone()
                        .map(|h| h.to_string())
                        .chain(once("all".to_string()))
                        .filter_map(move |h| {
                            if d == "all" && h == "all" {
                                None
                            } else {
                                Some(format!("{d}-{h}"))
                            }
                        })
                })
                .collect::<Vec<_>>(),
        ),
    }
}

impl Object for super::Restaurant {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        match key.as_str()? {
            "name" => Some(Value::from(&self.name)),
            "url" => Some(Value::from(&self.url)),
            "phone" => self.phone.as_ref().map(|phone| {
                let phone = phonenumber::parse(Some(phonenumber::country::US), phone)
                    .expect("phone number");
                context! {
                    display => Value::from(
                        phone
                            .format()
                            .mode(phonenumber::Mode::National)
                            .to_string(),
                    ),
                    url => Value::from(
                        phone
                            .format()
                            .mode(phonenumber::Mode::Rfc3966)
                            .to_string()
                    ),
                }
            }),
            "map_id" => Some(Value::from(&self.map_id)),
            "instagram_id" => Some(Value::from(&self.instagram_id)),
            "verified" => Some(Value::from_serialize(self.verified)),
            "description" => {
                if let super::Kind::HappyHour { description, .. } = &self.kind {
                    Some(Value::from_serialize(description))
                } else {
                    None
                }
            }
            "menu_url" => {
                if let super::Kind::HappyHour {
                    menu_url: Some(menu_url),
                    ..
                } = &self.kind
                {
                    Some(Value::from_serialize(menu_url))
                } else {
                    None
                }
            }
            "happytimes" => {
                if let super::Kind::HappyHour { happytimes, .. } = &self.kind {
                    let human_times: Vec<_> = happytimes
                        .as_human_readable()
                        .into_iter()
                        .map(|ht| {
                            context! {
                                description => ht.description,
                                data_attributes => ht.data_attributes,
                            }
                        })
                        .collect();
                    Some(context! {
                        data_attributes => happytimes.as_data_attributes(),
                        times => human_times,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Str(&[
            "name",
            "url",
            "map_id",
            "instagram_id",
            "verified",
            "description",
            "menu_url",
            "happytimes",
        ])
    }
}
