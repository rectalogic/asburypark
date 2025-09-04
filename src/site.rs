use crate::{
    model::{HappyTimes, Hours, Restaurants},
    ron_options,
};
use anyhow::{Context, Result, anyhow};
use minijinja::{AutoEscape, Environment, Value, context, value::ViaDeserialize};
use std::{
    fs::{self, OpenOptions},
    iter::once,
    path::{Path, PathBuf},
};

pub struct SiteGenerator<'a> {
    jinja: Environment<'a>,
    context: Value,
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

        jinja.add_global(
            "rangehours",
            Value::from_object((Hours::START_HOUR..=Hours::END_HOUR).collect::<Vec<_>>()),
        );
        jinja.add_global("rangedays", Value::from_object((0..=6).collect::<Vec<_>>()));
        jinja.add_global(
            "rangedayhours",
            Value::from_object(
                (0..=6)
                    .map(|d| d.to_string())
                    .chain(once("all".to_string()))
                    .flat_map(|d| {
                        (Hours::START_HOUR..=Hours::END_HOUR)
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
        );
        // XXX consider making HappyTimes a Value::Object
        jinja.add_function("restaurant_convert_happytimes", happytime_converter);

        Ok(Self {
            jinja,
            context: context!(restaurants => restaurants),
            site: site.to_owned(),
        })
    }

    pub fn build(&self, output: impl AsRef<Path>) -> Result<()> {
        let output = output.as_ref();
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
            if let Err(err) = template.render_to_write(&self.context, f) {
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

fn happytime_converter(happytimes: ViaDeserialize<HappyTimes>) -> Value {
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
    context! {
        restaurant_data_attributes => happytimes.as_data_attributes(),
        human_readable_times => human_times,
    }
}
