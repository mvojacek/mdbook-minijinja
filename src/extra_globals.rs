use crate::config::MiniJinjaConfig;
use mdbook::book::Chapter;
use mdbook::preprocess::PreprocessorContext;
use minijinja::value::{Enumerator, Object};
use minijinja::Value;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug)]
pub struct EnvironmentObject {
    vars: HashMap<&'static str, String>,
    keys: &'static [&'static str],
}

impl EnvironmentObject {
    pub fn new() -> Self {
        let vars: HashMap<&'static str, String> = env::vars_os()
            .filter_map(|(k, v)| {
                if let (Ok(k), Ok(v)) = (k.into_string(), v.into_string()) {
                    let k: &'static str = k.leak();
                    Some((k, v))
                } else {
                    None
                }
            })
            .collect();
        let keys: Vec<&'static str> = vars.keys().map(|k| *k).collect();

        EnvironmentObject {
            vars,
            keys: keys.leak(),
        }
    }
}

impl Object for EnvironmentObject {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        self.vars.get(key.as_str()?).map(Value::from)
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Str(self.keys)
    }
}

#[derive(Debug)]
pub struct ChapterObject {
    pub name: String,
    pub path: Option<String>,
    pub dir: Option<String>,
    pub source_path: Option<String>,
    pub source_dir: Option<String>,
}

impl From<&Chapter> for ChapterObject {
    fn from(chapter: &Chapter) -> Self {
        ChapterObject {
            name: chapter.name.clone(),
            path: chapter.path.as_ref().and_then(|p| p.to_str()).map(|p| p.to_string()),
            dir: chapter.path.as_ref().and_then(|p| p.parent()).and_then(|p| p.to_str()).map(|p| p.to_string()),
            source_path: chapter.source_path.as_ref().and_then(|p| p.to_str()).map(|p| p.to_string()),
            source_dir: chapter.source_path.as_ref().and_then(|p| p.parent()).and_then(|p| p.to_str()).map(|p| p.to_string()),
        }
    }
}

impl Object for ChapterObject {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        match key.as_str()? {
            "name" => Some(&self.name),
            "path" => self.path.as_ref(),
            "dir" => self.dir.as_ref(),
            "source_path" => self.source_path.as_ref(),
            "source_dir" => self.source_dir.as_ref(),
            _ => None,
        }.map(Value::from)
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Str(&["name", "path", "dir", "source_path", "source_dir"])
    }
}

#[derive(Debug)]
pub struct BookObject {
    pub root_dir: String,
    pub src_dir: String,
    pub template_dir: String,
    pub build_dir: String,
}

impl BookObject {
    pub fn new(ctx: &PreprocessorContext, config: &MiniJinjaConfig) -> Self {
        fn resolve_to_root(path: impl AsRef<Path>, root: impl AsRef<Path>) -> String {
            let path = path.as_ref();
            let root = root.as_ref();

            if path.is_absolute() {
                path.to_string_lossy().to_string()
            } else {
                root.join(path).to_string_lossy().to_string()
            }
        }

        let root = &ctx.root;

        BookObject {
            root_dir: root.to_string_lossy().to_string(),
            src_dir: resolve_to_root(&ctx.config.book.src, root),
            template_dir: resolve_to_root(&config.templates_dir, root),
            build_dir: resolve_to_root(&ctx.config.build.build_dir, root),
        }
    }
}

impl Object for BookObject {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        Some(Value::from(match key.as_str()? {
            "root_dir" => &self.root_dir,
            "src_dir" => &self.src_dir,
            "template_dir" => &self.template_dir,
            "build_dir" => &self.build_dir,
            _ => return None,
        }))
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Str(&["root_dir", "src_dir", "template_dir", "build_dir"])
    }
}

pub mod functions {
    use crate::extra_globals::{BookObject, ChapterObject};
    use log::{debug, error, info};
    use minijinja::value::{Kwargs, ViaDeserialize};
    use minijinja::{Environment, Error, ErrorKind, State, Value};
    use serde::Deserialize;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[derive(Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum RelativeType {
        Absolute,
        Root,
        Source,
        Template,
        Build,
        Chapter,
        ChapterBuild,
    }

    fn get_chapterobject(state: &State) -> Result<Arc<ChapterObject>, Error> {
        state.lookup("chapter").and_then(|x| x.downcast_object()).ok_or(
            Error::new(ErrorKind::UndefinedError, "'chapter' global not found or wrong type")
        )
    }

    fn get_bookobject(state: &State) -> Result<Arc<BookObject>, Error> {
        state.lookup("book").and_then(|x| x.downcast_object()).ok_or(
            Error::new(ErrorKind::UndefinedError, "'book' global not found or wrong type")
        )
    }

    fn resolve_path(state: &State, path: &str, relative_type: RelativeType) -> Result<PathBuf, Error> {
        use RelativeType::*;
        Ok(match relative_type {
            Absolute => PathBuf::from(path),
            Root => PathBuf::from(&get_bookobject(state)?.root_dir).join(path),
            Source => PathBuf::from(&get_bookobject(state)?.src_dir).join(path),
            Template => PathBuf::from(&get_bookobject(state)?.template_dir).join(path),
            Build => PathBuf::from(&get_bookobject(state)?.build_dir).join(path),
            Chapter => PathBuf::from(&get_bookobject(state)?.src_dir).join(
                get_chapterobject(state)?.source_dir.as_ref().ok_or(
                    Error::new(ErrorKind::UndefinedError, "'source_dir' not found in 'chapter'")
                )?
            ).join(path),
            ChapterBuild => PathBuf::from(&get_bookobject(state)?.build_dir).join(
                get_chapterobject(state)?.source_dir.as_ref().ok_or(
                    Error::new(ErrorKind::UndefinedError, "'source_dir' not found in 'chapter'")
                )?
            ).join(path),
        })
    }

    fn get_relative_type(kwargs: &Kwargs, key: &str) -> Result<Option<RelativeType>, Error> {
        let relative_type: Option<ViaDeserialize<RelativeType>> = kwargs.get(key)?;
        Ok(relative_type.map(|x| x.0))
    }

    fn file_exists(state: &State, filename: &str, kwargs: Kwargs) -> Result<Value, Error> {
        debug!("file_exists: filename: {:?}, kwargs: {:?}", filename, kwargs);
        let relative_type = get_relative_type(&kwargs, "rel")?.unwrap_or(RelativeType::Template);
        kwargs.assert_all_used()?;
        let path = resolve_path(state, filename, relative_type)?;
        debug!("checking if file exists: {:?}", path);
        Ok(std::fs::metadata(path).is_ok().into())
    }

    fn copy_file(state: &State, src: &str, dst: Option<&str>, kwargs: Kwargs) -> Result<Value, Error> {
        debug!("copy_file: src: {:?}, dst: {:?}, kwargs: {:?}", src, dst, kwargs);
        let rel_src = get_relative_type(&kwargs, "srcrel")?.unwrap_or(RelativeType::Template);
        let rel_dst = get_relative_type(&kwargs, "dstrel")?.unwrap_or(RelativeType::ChapterBuild);
        kwargs.assert_all_used()?;
        let src_path = resolve_path(state, src, rel_src)?;
        let dst_path = resolve_path(state, dst.unwrap_or(src), rel_dst)?;
        info!("copying file from {:?} to {:?}", src_path, dst_path);
        match std::fs::copy(src_path, dst_path) {
            Ok(_) => Ok(true.into()),
            Err(e) => Err(Error::new(ErrorKind::WriteFailure, format!("could not copy file: {}", e))),
        }
    }

    fn load_file(state: &State, filename: &str, kwargs: Kwargs) -> Result<Value, Error> {
        debug!("load_file: filename: {:?}, kwargs: {:?}", filename, kwargs);
        let relative_type = get_relative_type(&kwargs, "rel")?.unwrap_or(RelativeType::Template);
        kwargs.assert_all_used()?;
        let path = resolve_path(state, filename, relative_type)?;
        debug!("loading file: {:?}", path);
        match std::fs::read_to_string(path) {
            Ok(content) => Ok(content.into()),
            Err(e) => Err(Error::new(ErrorKind::InvalidOperation, format!("could not read file: {}", e))),
        }
    }

    pub fn add_functions(env: &mut Environment) {
        env.add_function("file_exists", file_exists);
        env.add_function("copy_file", copy_file);
        env.add_function("load_file", load_file);
    }
}