use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
use std::error::Error;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;

/// Alias for a `(String, fn(h: &Helper<'_, '_>, hb: &Handlebars<'_>, c: &Context, rc: &mut
/// RenderContext<'_, '_>, out: &mut dyn Output) -> HelperResult)`.
pub type HandlebarsHelper = (
    String,
    fn(
        h: &Helper,
        hb: &Handlebars,
        c: &Context,
        rc: &mut RenderContext,
        out: &mut dyn Output,
    ) -> Result<(), RenderError>,
);

/// A recipe for `render_pdf` which specifies an input template path, an output PDF path, data in
/// form of mapping (`Serialize`able) and an optional vector of `HandlebarsHelper`
#[derive(Clone)]
pub struct TemplateRecipe<'a, T: serde::Serialize> {
    pub template: &'a Path,
    pub output: &'a Path,
    pub data: &'a T,
    pub helpers: Option<Vec<HandlebarsHelper>>,
}

/// Outputs TeX from `TemplateRecipe`
pub fn prepare_tex<T: serde::Serialize>(
    recipe: &TemplateRecipe<T>,
) -> Result<String, Box<dyn Error>> {
    let mut hb_reg = Handlebars::new();
    hb_reg.register_escape_fn(|s| s.to_string());

    let template_name = "tex_template";

    if let Some(helpers) = &recipe.helpers {
        for h in helpers {
            let (n, f) = h;
            hb_reg.register_helper(n, Box::new(f));
        }
    }

    let tex_content = read_to_string(recipe.template).expect("Cannot read template file");

    hb_reg.register_template_string(template_name, tex_content)?;

    Ok(hb_reg.render(template_name, recipe.data)?)
}

/// Outputs PDF from `TemplateRecipe` using `tectonic::latex_to_pdf`
pub fn render_pdf<T: serde::Serialize>(recipe: &TemplateRecipe<T>) -> Result<(), Box<dyn Error>> {
    let tex = prepare_tex::<T>(recipe)?;

    let pdf_data: Vec<u8> = tectonic::latex_to_pdf(&tex)?;
    let mut file = File::create(recipe.output)?;
    file.write_all(&pdf_data)?;

    Ok(())
}

/// Outputs TeX and PDF from `TemplateRecipe` using `tectonic::latex_to_pdf`
pub fn render_tex<T: serde::Serialize>(
    recipe: &TemplateRecipe<T>,
    tex_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let tex = prepare_tex::<T>(recipe)?;

    let mut tex_file = File::create(tex_path)?;
    tex_file.write_all(tex.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_render_tex() {
        let latex_input = r#"
            \documentclass{article}
            \begin{document}
                Hello, {{foo}}!
            \end{document}
        "#;
        let latex_output = r#"
            \documentclass{article}
            \begin{document}
                Hello, boo!
            \end{document}
        "#;

        let dir = tempdir().expect("Temp dir cannot be created");

        let tex_path = dir.path().join("test.tex");
        let pdf_path = dir.path().join("test.pdf");

        {
            let mut file = File::create(&tex_path).expect("Temp TeX cannot be created");
            write!(file, "{}", latex_input).unwrap();
        }

        let mut data = HashMap::new();
        data.insert("foo", "boo");

        let t = TemplateRecipe {
            template: &tex_path,
            output: &pdf_path,
            data: &data,
            helpers: None,
        };

        let output = prepare_tex(&t);

        assert_eq!(output.unwrap(), latex_output);
    }

    #[test]
    fn test_render_pdf() {
        let latex = r#"
            \documentclass{article}
            \begin{document}
                Hello, {{foo}}!
            \end{document}
        "#;

        let dir = tempdir().expect("Temp dir cannot be created");

        let tex_path = dir.path().join("test.tex");
        let pdf_path = dir.path().join("test.pdf");

        {
            let mut file = File::create(&tex_path).expect("Temp TeX cannot be created");
            write!(file, "{}", latex).unwrap();
        }

        let mut data = HashMap::new();
        data.insert("foo", "boo");

        let t = TemplateRecipe {
            template: &tex_path,
            output: &pdf_path,
            data: &data,
            helpers: None,
        };

        let _ = render_pdf(&t);

        {
            let file = File::open(&pdf_path).expect("Temp TeX cannot be opened");
            assert_eq!(file.metadata().unwrap().len(), 2767);
        }
    }

    #[test]
    fn test_render_html_like() {
        let latex_input = "Hello, {{name}}!";
        let data = HashMap::from([("name", "<&%#>".to_owned())]);
        
        let dir = tempdir().expect("Temp dir cannot be created");

        let tex_path = dir.path().join("test.tex");
        let pdf_path = dir.path().join("test.pdf");

        {
            let mut file = File::create(&tex_path).expect("Temp TeX cannot be created");
            write!(file, "{}", latex_input).unwrap();
        }

        let t = TemplateRecipe {
            template: &tex_path,
            output: &pdf_path,
            data: &data,
            helpers: None,
        };

        let output = prepare_tex(&t).unwrap();
        assert_eq!(output, "Hello, <&%#>!");
    }
}
