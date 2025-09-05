use crate::{
    model::{Restaurants, restaurants_value},
    ron_options,
};
use anyhow::{Context, Result, anyhow};
use minijinja::{AutoEscape, Environment, context};
use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
};

pub struct SiteGenerator<'a> {
    jinja: Environment<'a>,
    restaurants: Restaurants,
    site: PathBuf,
}

impl<'a> SiteGenerator<'a> {
    pub fn new(site: impl AsRef<Path>) -> Result<Self> {
        let site = site.as_ref();
        let ronpath = site.join("_data/restaurants.ron");
        let restaurants = fs::File::open(&ronpath).context(format!("{ronpath:?}"))?;
        let restaurants: Restaurants = ron_options().from_reader(restaurants)?;

        let mut jinja = Environment::new();
        jinja.set_auto_escape_callback(|_| AutoEscape::None);

        let templates = &site.join("_templates");
        visit_files(templates, &mut |path: &Path| -> Result<()> {
            let filename = path.strip_prefix(templates)?;
            jinja.add_template_owned(
                filename
                    .to_str()
                    .ok_or(anyhow!("Invalid filename"))?
                    .to_owned(),
                fs::read_to_string(path)?,
            )?;
            Ok(())
        })?;

        Ok(Self {
            jinja,
            restaurants,
            site: site.to_owned(),
        })
    }

    pub fn build(self, output: impl AsRef<Path>) -> Result<()> {
        let output = output.as_ref();

        let json_path = output.join("restaurant.json");
        create_parent_dirs(&json_path)
            .with_context(|| format!("create parents {}", json_path.display()))?;
        let json_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&json_path)
            .with_context(|| format!("open failed {}", json_path.display()))?;
        serde_json::to_writer_pretty(json_file, &self.restaurants)?;

        let context = context! { restaurants => restaurants_value(self.restaurants) };

        for (name, template) in self.jinja.templates() {
            let output_path = output.join(name);
            create_parent_dirs(&output_path)
                .with_context(|| format!("create parents {}", output_path.display()))?;
            let f = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&output_path)
                .with_context(|| format!("open failed {}", output_path.display()))?;
            if let Err(err) = template.render_to_write(&context, f) {
                eprintln!("Render failed: {err:?}");
                return Err(anyhow!("Render failed"));
            }
        }

        visit_files(&self.site, &mut |path: &Path| -> Result<()> {
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

fn visit_files<F>(dir: &Path, cb: &mut F) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
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
