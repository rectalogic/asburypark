use crate::model::{HappyTimes, Restaurants};
use anyhow::{Context, Result, anyhow};
use ron::de::from_reader;
use std::{collections::HashMap, fs::File, path::Path};
use tera::{Context as TeraContext, Tera};

pub struct SiteGenerator {
    tera: Tera,
    context: TeraContext,
}

impl SiteGenerator {
    pub fn new(site: impl AsRef<Path>) -> Result<Self> {
        let ronpath = site.as_ref().join("data/restaurants.ron");
        let restaurants = File::open(&ronpath).context(format!("{ronpath:?}"))?;
        let restaurants: Restaurants = from_reader(restaurants)?;
        let mut context = TeraContext::new();
        context.insert("restaurants", &restaurants);
        let mut tera = Tera::new(
            site.as_ref()
                .join("templates/**/*.html")
                .to_str()
                .ok_or(anyhow!("invalid template path"))?,
        )?;
        tera.register_function("restaurant_data_attributes", HappyTimesDataAttributes);
        Ok(Self { tera, context })
    }

    pub fn render(&self, template_name: &str) -> Result<String> {
        Ok(self.tera.render(template_name, &self.context)?)
    }
}

struct HappyTimesDataAttributes;

impl tera::Function for HappyTimesDataAttributes {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        match args.get("happytimes") {
            Some(happytimes) => {
                let happytimes = serde_json::from_value::<HappyTimes>(happytimes.clone())?;
                Ok(happytimes.to_css_data_attributes().into())
            }
            None => Err("Missing argument 'happytimes'".into()),
        }
    }

    fn is_safe(&self) -> bool {
        true
    }
}
