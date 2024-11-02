use mdbook::{
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use serde::Serialize;

pub struct MiniJinjaPreprocessor;

impl Preprocessor for MiniJinjaPreprocessor {
    fn name(&self) -> &'static str {
        "minijinja"
    }

    fn run(
        &self,
        ctx: &PreprocessorContext,
        mut book: mdbook::book::Book,
    ) -> anyhow::Result<mdbook::book::Book> {
        let env = {
            let mut env = minijinja::Environment::new();
            env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
            env
        };

        let conf = ctx
            .config
            .get_preprocessor(self.name())
            .ok_or_else(|| anyhow::anyhow!("missing config section for {}", self.name()))?;

        let vars = conf.get("variables");

        book.for_each_mut(|item| match item {
            BookItem::Chapter(c) => {
                eval_in_place(&env, &mut c.name, vars);
                eval_in_place(&env, &mut c.content, vars);
            }
            BookItem::Separator => {}
            BookItem::PartTitle(ref mut title) => eval_in_place(&env, title, vars),
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
