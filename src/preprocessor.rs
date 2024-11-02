use mdbook::{
    book::{Book, SummaryItem},
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use serde::Serialize;

use crate::config;

pub struct MiniJinjaPreprocessor;

impl Preprocessor for MiniJinjaPreprocessor {
    fn name(&self) -> &'static str {
        "minijinja"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> anyhow::Result<Book> {
        let conf: Option<config::MiniJinjaConfig> = ctx
            .config
            .get_deserialized_opt(format!("preprocessor.{}", self.name()))?;

        let Some(conf) = conf else {
            anyhow::bail!("missing config section for {}", self.name())
        };

        log::debug!("{conf:#?}");

        let env = conf.create_env(&ctx.root);

        // XXX: mdBook has already loaded the summary by the time we get here,
        // so we need to load it ourselves, evaluate it as a template, and then
        // try to figure out how that should modify what mdbook loaded.
        //
        // This doesn't really fully work: we can support basic templated
        // values in chapter names, and conditionally included/excluded
        // chapters, but fully general jinja templates aren't supported.
        let mut summary_text = std::fs::read_to_string(ctx.config.book.src.join("SUMMARY.md"))?;
        eval_in_place(&env, &mut summary_text, &conf.variables);

        let summary = mdbook::book::parse_summary(&summary_text)?;
        let summary_names = summary
            .prefix_chapters
            .iter()
            .chain(summary.numbered_chapters.iter())
            .chain(summary.suffix_chapters.iter())
            .filter_map(|c| match c {
                SummaryItem::Link(l) => Some(l.name.clone()),
                _ => None,
            })
            .collect::<std::collections::HashSet<_>>();

        // Filter out sections that should get dropped after evaluating the
        // summary as a template.
        book.sections = book
            .sections
            .iter()
            .filter_map(|item| match item {
                BookItem::Chapter(c) => match env.render_str(&c.name, &conf.variables) {
                    Ok(name) => {
                        if summary_names.contains(&name) {
                            Some(item)
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        log_jinja_err(&e);
                        None
                    }
                },
                _ => Some(item),
            })
            .cloned()
            .collect::<Vec<_>>();

        book.for_each_mut(|item| match item {
            BookItem::Chapter(c) => {
                eval_in_place(&env, &mut c.name, &conf.variables);
                eval_in_place(&env, &mut c.content, &conf.variables);
            }
            BookItem::Separator => {}
            BookItem::PartTitle(ref mut title) => eval_in_place(&env, title, &conf.variables),
        });

        Ok(book)
    }
}

fn eval_in_place(env: &minijinja::Environment, s: &mut String, vars: impl Serialize) {
    match env.render_str(s, vars) {
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
