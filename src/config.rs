use minijinja::Environment;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MiniJinjaConfig {
    pub variables: toml::Table,
    #[serde(default)]
    pub undefined_behavior: UndefinedBehavior,
    #[serde(default = "MiniJinjaConfig::default_templates_dir")]
    pub templates_dir: PathBuf,
}

impl MiniJinjaConfig {
    pub fn create_env<'source>(&self, root: &PathBuf) -> Environment<'source> {
        let mut env = Environment::new();
        env.set_undefined_behavior(self.undefined_behavior.into());

        let templates_dir = if self.templates_dir.is_absolute() {
            self.templates_dir.clone()
        } else {
            root.join(&self.templates_dir)
        };
        log::info!("loading templates from {}", templates_dir.display());

        env.set_loader(minijinja::path_loader(templates_dir));
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
