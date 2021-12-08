use std::error::Error;
use std::path::Path;
use std::io::Write;
use std::fs::File;
use handlebars::{
    Handlebars,
    Helper,
    Context,
    RenderContext,
    Output,
    HelperResult
};

/// Alias for a `(String, fn(h: &Helper<'_, '_>, hb: &Handlebars<'_>, c: &Context, rc: &mut
/// RenderContext<'_, '_>, out: &mut dyn Output) -> HelperResult)`.
pub type HandlebarsHelper =
    (String,
     fn(h: &Helper,
        hb: &Handlebars,
        c: &Context,
        rc: &mut RenderContext,
        out: &mut dyn Output)
     -> HelperResult);

/// A recipe for `render_pdf` which specifies an input template path, an output PDF path, data in
/// form of mapping (`Serialize`able) and an optional vector of `HandlebarsHelper`
#[derive(Clone)]
pub struct TemplateRecipe<'a, T: serde::Serialize> {
    pub template: &'a Path,
    pub output: &'a Path,
    pub data: &'a T,
    pub helpers: Option<Vec<HandlebarsHelper>>,
}

/// Outputs PDF from `TemplateRecipe` using `tectonic::latex_to_pdf`
pub fn render_pdf<T: serde::Serialize>(recipe: &TemplateRecipe<T>) -> Result<(), Box<dyn Error>> {
    let mut handlebars = Handlebars::new();
    let template_name = "tex_template";

    if let Some(helpers) = &recipe.helpers {
        for h in helpers {
            let (n, f) = h;
            handlebars.register_helper(n, Box::new(f));
        }
    }

    handlebars.register_template_file(template_name,
                                      recipe.template.to_str().unwrap())?;

    let latex = handlebars.render(template_name, recipe.data)?;
    let pdf_data: Vec<u8> = tectonic::latex_to_pdf(&latex)?;
    let mut file = File::create(&recipe.output)?;
    file.write_all(&pdf_data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use std::fs::File;
    use std::collections::HashMap;
    use std::io::Write;
    use super::{render_pdf, TemplateRecipe};

    #[test]
    fn render() {

        let latex = r#"
            \documentclass{article}
            \begin{document}
                Hello, {{foo}}!
            \end{document}
        "#;

        let mut dir = tempdir().expect("Temp dir cannot be created");

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
            helpers: None
        };

        let _ = render_pdf(&t);

        {
            let file = File::open(&pdf_path).expect("Temp TeX cannot be created");
            assert_eq!(file.metadata().unwrap().len(), 2761);
        }
    }
}
