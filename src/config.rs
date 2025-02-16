use minijinja::{Environment, Value};
use std::path::PathBuf;

use serde::Deserialize;
use crate::extra_globals::EnvironmentObject;

#[derive(Debug, Deserialize)]
pub struct MiniJinjaConfig {
    /// Whether we should preprocess SUMMARY.md.
    #[serde(default)]
    pub preprocess_summary: bool,

    /// Variables to be passed to the minijinja environment.
    pub variables: toml::Table,

    /// Undefined behavior setting for minijinja.
    #[serde(default)]
    pub undefined_behavior: UndefinedBehavior,

    /// Templates directory for minijinja.
    #[serde(default = "MiniJinjaConfig::default_templates_dir")]
    pub templates_dir: PathBuf,

    #[serde(default)]
    pub prelude_string: String,

    #[serde(default)]
    pub global_env: bool,
}

impl MiniJinjaConfig {
    /// Create a new minijinja::Environment based on the configuration.
    pub fn create_env<'source>(&self, root: &PathBuf) -> Environment<'source> {
        let mut env = Environment::new();
        env.set_undefined_behavior(self.undefined_behavior.into());

        let templates_dir = if self.templates_dir.is_absolute() {
            self.templates_dir.clone()
        } else {
            root.join(&self.templates_dir)
        };

        log::debug!("loading templates from {}", templates_dir.display());

        env.set_loader(minijinja::path_loader(templates_dir));

        if self.global_env {
            env.add_global("env", Value::from_object(EnvironmentObject::new()));
        }

        env
    }

    fn default_templates_dir() -> PathBuf {
        PathBuf::from("templates")
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum UndefinedBehavior {
    #[serde(alias = "lenient")]
    Lenient,
    #[serde(alias = "chainable")]
    Chainable,
    #[serde(alias = "strict")]
    Strict,
}

impl Default for UndefinedBehavior {
    fn default() -> Self {
        Self::Strict
    }
}

impl Into<minijinja::UndefinedBehavior> for UndefinedBehavior {
    fn into(self) -> minijinja::UndefinedBehavior {
        match self {
            Self::Lenient => minijinja::UndefinedBehavior::Lenient,
            Self::Strict => minijinja::UndefinedBehavior::Strict,
            Self::Chainable => minijinja::UndefinedBehavior::Chainable,
        }
    }
}
