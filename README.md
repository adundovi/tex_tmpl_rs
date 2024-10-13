# tex_tmpl_rs

A small wrapper library for rendering a [LaTeX](https://www.latex-project.org/) / [Handlebars](https://github.com/sunng87/handlebars-rust) template into a PDF document using [Tectonic](https://tectonic-typesetting.github.io).

## Example

```tex
\documentclass{article}
\begin{document}
    Hello, {{foo}}!
\end{document}
```

```rust
let mut data = HashMap::new();
data.insert("foo", "boo");

let t = TemplateRecipe {
    template: &tex_path,
    output: &pdf_path,
    data: &data,
    helpers: None,
};

let _ = render_pdf(&t);
```

## Dependencies

Fedora:

```sh
dnf install freetype-devel graphite2-devel libicu-devel fontconfig-devel gcc-c++ libpng-devel
```

### Optional

```sh
cargo install -F external-harfbuzz tectonic
```
