# mdbook-minijinja

mdbook-minijinja is an [mdbook][mdbook] [preprocessor][mdbook-preprocessor]
that evaluates the files in your book as [minijinja][minijinja]
templates. Template features are fully supported inside book chapters.  Limited
template features are available in `SUMMARY.md` (see below).

[mdbook]: https://rust-lang.github.io/mdBook
[mdbook-preprocessor]: https://rust-lang.github.io/mdBook/format/configuration/preprocessors.html
[minijinja]: https://docs.rs/minijinja/latest/minijinja/

## Example Configuration

```toml
# book.toml
[preprocessor.minijinja]

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
# interpreted relative to the path containing book.toml.
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

## SUMMARY.md Limitations

The structure of an mdbook is defined by the top-level
[SUMMARY.md](https://rust-lang.github.io/mdBook/format/summary.html) file,
which contains a list of the book's chapters and titles. mdbook only invokes
preprocessors after it has already parsed and evaluated SUMMARY.md. This means
mdbook-minijinja can only support a limited set of jinja template operations in
SUMMARY.md:

- ✅ Simple if/else conditionals to enable or disable chapters based on
  variables are supported.
- ✅ Template expressions within chapter and part titles are supported.
- ❌ `{% include %}` or other template expansions that evaluate to new book
  chapters are not supported. All book chapters must be present in the
  SUMMARY.md source.

For an example of supported functionality, see the [example
book](./example-book/src/SUMMARY.md).
