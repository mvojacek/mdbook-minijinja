use mdbook::{
    book::Book,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem, MDBook,
};
use serde::Serialize;

use crate::config;
use crate::config::MiniJinjaConfig;

pub struct MiniJinjaPreprocessor;

impl Preprocessor for MiniJinjaPreprocessor {
    fn name(&self) -> &'static str {
        "minijinja"
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> anyhow::Result<Book> {
        let conf: Option<config::MiniJinjaConfig> = ctx
            .config
            .get_deserialized_opt(format!("preprocessor.{}", self.name()))?;

        let Some(conf) = conf else {
            anyhow::bail!("missing config section for {}", self.name())
        };

        log::trace!("{conf:#?}");

        let env = conf.create_env(&ctx.root);

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
            eval_in_place(&conf, &env, &mut summary_text, &conf.variables);
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
                eval_in_place(&conf,&env, &mut c.name, &conf.variables);
                eval_in_place(&conf, &env, &mut c.content, &conf.variables);
            }
            BookItem::Separator => {}
            BookItem::PartTitle(ref mut title) => eval_in_place(&conf, &env, title, &conf.variables),
        });

        Ok(book)
    }
}

fn eval_in_place(conf: &MiniJinjaConfig, env: &minijinja::Environment, s: &mut String, vars: impl Serialize) {
    let s_with_prelude = if conf.prelude_string.is_empty() {
        &*s
    } else {
        &*format!("{}\n{}", conf.prelude_string, s)
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
