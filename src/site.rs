use crate::model::{HappyTimes, Restaurants};
use anyhow::{Context, Result, anyhow};
use ron::de::from_reader;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tera::{Context as TeraContext, Tera};

pub struct SiteGenerator {
    tera: Tera,
    context: TeraContext,
    site: PathBuf,
}

impl SiteGenerator {
    pub fn new(site: impl AsRef<Path>) -> Result<Self> {
        let site = site.as_ref();
        let ronpath = site.join("_data/restaurants.ron");
        let restaurants = fs::File::open(&ronpath).context(format!("{ronpath:?}"))?;
        let restaurants: Restaurants = from_reader(restaurants)?;
        let mut context = TeraContext::new();
        context.insert("restaurants", &restaurants);
        let mut tera = Tera::new(
            site.join("_templates/**/*")
                .to_str()
                .ok_or(anyhow!("invalid template path"))?,
        )?;
        tera.autoescape_on(vec![]);
        tera.register_function("restaurant_convert_happytimes", HappyTimesConverter);
        Ok(Self {
            tera,
            context,
            site: site.to_owned(),
        })
    }

    pub fn build(&self, output: impl AsRef<Path>) -> Result<()> {
        let output = output.as_ref();
        for template in self.tera.get_template_names() {
            if template.ends_with(".macro") {
                continue;
            }
            let rendered = self.tera.render(template, &self.context)?;
            let output_path = output.join(template);
            create_parent_dirs(&output_path)?;
            fs::write(output_path, rendered)?;
        }

        visit_files(&self.site, &|path: &Path| -> Result<()> {
            let filename = path.strip_prefix(&self.site)?;
            if filename.starts_with("_templates") || filename.starts_with("_data") {
                return Ok(());
            }
            let dest = output.join(filename);
            copy_path(path, dest)?;
            Ok(())
        })?;

        Ok(())
    }
}

fn visit_files<F>(dir: &Path, cb: &F) -> Result<()>
where
    F: Fn(&Path) -> Result<()>,
{
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_files(&path, cb)?;
            } else {
                cb(path.as_ref())?;
            }
        }
    }
    Ok(())
}

fn create_parent_dirs(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn copy_path<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    let to = to.as_ref();
    create_parent_dirs(to)?;
    fs::copy(from, to)?;
    Ok(())
}

struct HappyTimesConverter;

impl tera::Function for HappyTimesConverter {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        match args.get("happytimes") {
            Some(happytimes) => {
                let happytimes = serde_json::from_value::<HappyTimes>(happytimes.clone())?;
                let mut map = tera::Map::new();
                map.insert(
                    "data_attributes".into(),
                    happytimes.as_data_attributes().into(),
                );
                map.insert(
                    "human_readable".into(),
                    happytimes.as_human_readable().into(),
                );
                Ok(tera::Value::Object(map))
            }
            None => Err("Missing argument 'happytimes'".into()),
        }
    }

    fn is_safe(&self) -> bool {
        true
    }
}
