# mdbook-minijinja

mdbook-minijinja is an [mdbook][mdbook] [preprocessor][mdbook-preprocessor]
that evaluates the files in your book as [minijinja][minijinja] templates.

See the [example
book](https://github.com/ssanderson/mdbook-minijinja/tree/main/example-book)
for a full example.

[mdbook]: https://rust-lang.github.io/mdBook
[mdbook-preprocessor]: https://rust-lang.github.io/mdBook/format/configuration/preprocessors.html
[minijinja]: https://docs.rs/minijinja/latest/minijinja/

## Example Configuration

```toml
# book.toml
[preprocessor.minijinja]

# Whether or not mdbook-minijinja should evaluate SUMMARY.md
# as a template. If this is true, mdbook-minijinja will reload SUMMARY.md,
# evaluate it as a template, and then reload book chapters from the
# re-parsed SUMMARY.md. This discards the effects of any preprocessors
# that ran before mdbook-minijinja, so mdbook-minijinja should be configured
# as the first preprocessor if summary preprocessing is enabled. Use
# the `before` key to configure preprocessor order.
#
# Default value is false.
preprocess_summary = true

# Configure mdbook-minijinja to run before other preprocessors.
#
# "index" and "links" are built-in preprocessors run by mdbook by default. If you
# have other preprocessors enabled, you may want to include them here as well.
before = ["index", "links"]

# Configure behavior of evaluating undefined variables in minijinja.
#
# Options are "strict", "lenient", or "chained".
#
# See https://docs.rs/minijinja/latest/minijinja/enum.UndefinedBehavior.html
# for more details.
#
# Default value is "strict".
undefined_behavior = "strict"

# Path to a directory containing minijinja templates. Minijinja import and
# include directives will look for templates here.
#
# If this path is absolute, it is used as-is. If it is relative, it is
# interpreted relative to the directory containing book.toml.
#
# See https://docs.rs/minijinja/latest/minijinja/fn.path_loader.html for more
# details.
#
# Default value is "templates".
templates = "templates"

# Variables defined in this section will be available for use in templates.
[preprocessor.minijinja.variables]
my_var = "my_var_value"
chapter_1_name = "Cool Chapter 1"
part_1_name = "Cool Part 1"
part_2_name = "Cool Part 2"
condition_true = true
condition_false = false
list_of_strings = ["foo", "bar", "buzz"]
partial_chapter_name = "Partial"
```

## Preprocessing SUMMARY.md

The structure of an mdbook is defined by the top-level
[SUMMARY.md](https://rust-lang.github.io/mdBook/format/summary.html) file,
which contains a list of the book's sections and chapters.

MDBook only invokes preprocessors after SUMMARY.md has already been loaded and
parsed. This creates a challenge for preprocessors like mdbook-minijinja that
want to preprocess SUMMARY.md,

To work around the above, if `preprocess_summary` is set to `true`,
mdbook-minijinja reloads SUMMARY.md and evaluates it as a minijinja
template. We then reload all chapters referenced by the updated
SUMMARY.md. This allows SUMMARY.md to be evaluated as a minijinja template, but
it means that **we discard the results of any preprocessors that ran before
mdbook-minijinja**.

If you enable summary preprocessing, we recommend configuring mdbook-minijinja
as your first preprocessor using the [before and
after](https://rust-lang.github.io/mdBook/format/configuration/preprocessors.html#require-a-certain-order)
configuration values. See the example configuration above for an example.
