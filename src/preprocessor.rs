use mdbook::{
    book::Book,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem, MDBook,
};
use minijinja::Value;
use serde::Serialize;

use crate::config::MiniJinjaConfig;
use crate::extra_globals;
use crate::extra_globals::{BookObject, ChapterObject, EnvironmentObject};

pub struct MiniJinjaPreprocessor;

impl Preprocessor for MiniJinjaPreprocessor {
    fn name(&self) -> &'static str {
        "minijinja"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> anyhow::Result<Book> {
        let conf: Option<MiniJinjaConfig> = ctx
            .config
            .get_deserialized_opt(format!("preprocessor.{}", self.name()))?;

        let Some(conf) = conf else {
            anyhow::bail!("missing config section for {}", self.name())
        };

        log::trace!("{conf:#?}");

        let mut env = conf.create_env(&ctx.root);

        let book_object = BookObject::new(&ctx, &conf);
        env.add_global("book", Value::from_object(book_object));

        if conf.global_env {
            env.add_global("env", Value::from_object(EnvironmentObject::new()));
        }

        extra_globals::functions::add_functions(&mut env);

        let mut book = if conf.preprocess_summary {
            // mdBook has already loaded the summary by the time we get here, so we
            // need to reload it ourselves, evaluate it as a template, and then
            // replace what mdbook loaded with our own evaluated templates.
            //
            // This discards the output of any preprocessors that ran before us, so
            // mdbook-minijinja should be configured as the first preprocessor.
            let summary_path = ctx.config.book.src.join("SUMMARY.md");
            log::info!("reloading summary from {}", summary_path.display());

            let mut summary_text = std::fs::read_to_string(summary_path)?;
            eval_in_place(&conf, &env, &mut summary_text, &conf.variables, false);
            let summary = mdbook::book::parse_summary(&summary_text)?;

            let MDBook { book, .. } = MDBook::load_with_config_and_summary(
                ctx.root.clone(),
                ctx.config.clone(),
                summary,
            )?;
            book
        } else {
            log::info!("skipping preprocessing of SUMMARY.md because preprocess_summary is false");
            book
        };

        book.for_each_mut(|item| match item {
            BookItem::Chapter(c) => {
                env.add_global("chapter", Value::from_object(ChapterObject::from(&*c)));
                eval_in_place(&conf, &env, &mut c.name, &conf.variables, false);
                eval_in_place(&conf, &env, &mut c.content, &conf.variables, true);
            }
            BookItem::Separator => {}
            BookItem::PartTitle(ref mut title) => eval_in_place(&conf, &env, title, &conf.variables, false),
        });

        Ok(book)
    }
}

fn eval_in_place(conf: &MiniJinjaConfig, env: &minijinja::Environment, s: &mut String, vars: impl Serialize, prelude: bool) {
    let s_with_prelude = if prelude && !conf.prelude_string.is_empty() {
        &*format!("{}\n{}", conf.prelude_string, s)
    } else {
        &*s
    };

    match env.render_str(s_with_prelude, vars) {
        Ok(res) => {
            *s = res;
        }
        Err(e) => {
            log_jinja_err(&e);
            return;
        }
    };
}

fn log_jinja_err(err: &minijinja::Error) {
    log::error!("Could not render template: {:#}", err);
    // render causes as well
    let mut err = &err as &dyn std::error::Error;
    while let Some(next_err) = err.source() {
        log::error!("caused by: {:#}", next_err);
        err = next_err;
    }
}
