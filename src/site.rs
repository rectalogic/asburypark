use crate::model::Restaurants;
use anyhow::{Context, Result, anyhow};
use ron::de::from_reader;
use std::{fs::File, path::Path};
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
        Ok(Self {
            tera: Tera::new(
                site.as_ref()
                    .join("templates/**/*.html")
                    .to_str()
                    .ok_or(anyhow!("invalid template path"))?,
            )?,
            context,
        })
    }

    pub fn render(&self, template_name: &str) -> Result<String> {
        Ok(self.tera.render(template_name, &self.context)?)
    }
}
