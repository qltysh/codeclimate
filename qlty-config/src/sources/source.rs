use super::SourcesList;
use crate::config::Builder;
use crate::{QltyConfig, TomlMerge};
use anyhow::{Context, Result};
use config::File;
use globset::{Glob, GlobSetBuilder};
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use tracing::trace;

const SOURCE_PARSE_ERROR: &str = r#"There was an error reading configuration from one of your declared Sources.

Please make sure you are using the latest version of the CLI with `qlty upgrade`.

Also, please make sure you are specifying the latest source tag in your qlty.toml file.

For more information, please visit: https://qlty.io/docs/troubleshooting/source-parse-error"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: PathBuf,
    pub contents: String,
}

impl SourceFile {
    pub fn write_to(&self, path: &Path) -> Result<()> {
        std::fs::write(path, &self.contents).with_context(|| {
            format!(
                "Could not write the plugin configuration to {}",
                path.display()
            )
        })
    }
}

pub trait SourceFetch: Debug + Send + Sync {
    fn fetch(&self) -> Result<()>;
    fn clone_box(&self) -> Box<dyn SourceFetch>;
    fn sources(&self) -> Vec<Box<dyn Source>> {
        vec![]
    }
}

impl Clone for Box<dyn SourceFetch> {
    fn clone(&self) -> Box<dyn SourceFetch> {
        SourceFetch::clone_box(self.as_ref())
    }
}

impl Default for Box<dyn SourceFetch> {
    fn default() -> Box<dyn SourceFetch> {
        Box::<SourcesList>::default()
    }
}

pub trait Source: SourceFetch {
    fn plugin_tomls(&self) -> Result<Vec<SourceFile>> {
        let mut globset_builder = GlobSetBuilder::new();

        for pattern in vec![
            "*/linters/*/plugin.toml",
            "*/plugins/linters/*/plugin.toml",
            "linters/*/plugin.toml",
            "plugins/linters/*/plugin.toml",
        ] {
            globset_builder.add(Glob::new(&pattern)?);
        }

        let globset = globset_builder.build()?;

        Ok(self
            .files()?
            .into_iter()
            .filter(|file| {
                let is_match = globset.is_match(&file.path);
                dbg!(&file.path, is_match);
                is_match
            })
            .collect::<Vec<SourceFile>>())
    }

    fn files(&self) -> Result<Vec<SourceFile>>;

    fn get_config_file(&self, plugin_name: &str, config_file: &Path) -> Result<Option<SourceFile>> {
        let candidates = vec![
            PathBuf::from("plugins/linters")
                .join(plugin_name)
                .join(config_file),
            PathBuf::from("linters").join(plugin_name).join(config_file),
        ];

        for candidate in candidates {
            if let Some(file) = self.get_file(&candidate)? {
                return Ok(Some(file));
            }
        }

        Ok(None)
    }

    fn get_file(&self, file_name: &Path) -> Result<Option<SourceFile>>;

    fn toml(&self) -> Result<toml::Value> {
        let mut toml: toml::Value = toml::Value::Table(toml::value::Table::new());

        for plugin_toml in self.plugin_tomls()?.iter() {
            trace!("Loading plugin config from {}", plugin_toml.path.display());

            let contents_toml = plugin_toml
                .contents
                .parse::<toml::Value>()
                .with_context(|| format!("Could not parse {}", plugin_toml.path.display()))?;

            Builder::validate_toml(&plugin_toml.path, contents_toml.clone())
                .with_context(|| SOURCE_PARSE_ERROR)?;

            toml = TomlMerge::merge(toml, contents_toml).unwrap();
        }

        // dbg!(&self, &toml);

        Ok(toml)
    }

    fn build_config(&self) -> Result<QltyConfig> {
        let toml_string = toml::to_string(&self.toml()?).unwrap();
        let file = File::from_str(&toml_string, config::FileFormat::Toml);
        let builder = config::Config::builder().add_source(file);
        builder
            .build()?
            .try_deserialize()
            .context("Could not process the plugin configuration")
    }

    fn clone_box(&self) -> Box<dyn Source>;
}

impl Clone for Box<dyn Source> {
    fn clone(&self) -> Box<dyn Source> {
        Source::clone_box(self.as_ref())
    }
}
