use clap::{Arg, Command};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use semver::{Version, VersionReq};

mod preprocessor;
mod config;
mod extra_globals;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pre = preprocessor::MiniJinjaPreprocessor;
    let cmd = Command::new("mdbook-minijinja")
        .about("An mdbook preprocessor that adds support for jinja2 templates.")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        );

    if let Some(sub_args) = cmd.get_matches().subcommand_matches("supports") {
        sub_args
            .get_one::<String>("renderer")
            .map(|_| ())
            .ok_or_else(|| anyhow::anyhow!("missing expected renderer argument"))
    } else {
        let (ctx, book) = CmdPreprocessor::parse_input(std::io::stdin())?;
        let book_version = Version::parse(&ctx.mdbook_version)?;
        let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

        if !version_req.matches(&book_version) {
            log::warn!(
                "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
                pre.name(),
                mdbook::MDBOOK_VERSION,
                ctx.mdbook_version
            );
        }

        let result = pre.run(&ctx, book)?;

        Ok(serde_json::to_writer(std::io::stdout(), &result)?)
    }
}
