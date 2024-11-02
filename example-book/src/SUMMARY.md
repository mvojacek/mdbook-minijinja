# Summary

# {{ part_1_name }}

- [{{chapter_1_name}}](./chapter_1.md)
- [Chapter 2](./chapter_2.md)
{% if condition_true %}
- [Conditionally Enabled Chapter](./enabled.md)
{% endif %}
{% if condition_false %}
- [Conditionally Disabled Chapter](./disabled.md)
{% endif %}

# {{ part_2_name }}

- [Chapter {{ partial_chapter_name }}](./partial.md)

# Templates

- [Uses Templates](./templates.md)
