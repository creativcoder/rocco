//! rocco is a [http://ashkenas.com/docco](http://ashkenas.com/docco) inspired literate programmming library
//! It produces an HTML document that displays your comments intermingled with your code.
//! All prose is passed through Markdown (using [comrak](https://crates.io/comrak)),
//! and code is passed through prism.js syntax highlighting.
//!
//! Rocco has a simple API:
//!```no_run
//! use rocco::Docco;
//! use std::path::PathBuf;
//!
//! let input_source = PathBuf::from("tests/samples/source.rs");
//! let output = PathBuf::from("source.html");
//! let mut docco = Docco::new(input_source, Some(output)).unwrap();
//! docco.parse().unwrap();
//! docco.render().unwrap();
//! ```

mod error;

use error::Error;
use once_cell::sync::Lazy;
use ramhorns::{Content, Template};
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Lines};
use std::iter::Peekable;
use std::path::PathBuf;

#[derive(Debug, Content, serde::Serialize, serde::Deserialize)]
pub struct Language {
    // name of language
    name: String,
    // the delimiter which denotes a comment
    comment: String,
}

static LANGUAGES: Lazy<HashMap<&'static str, Language>> = Lazy::new(|| {
    let lang_json = include_str!("assets/languages.json");
    serde_json::from_str(lang_json).expect("Language map initialization failed")
});

#[derive(Content, Debug)]
pub struct Section {
    num: usize,
    docs_html: String,
    code_html: String,
}

#[derive(Content)]
pub struct Docco {
    sections: Vec<Section>,
    css: String,
    html: String,
    filename: String,
    output: String,
    extension: String,
    language: String,
    doc_symbol: String,
}

impl Docco {
    pub fn new(source: PathBuf, output: Option<PathBuf>) -> Result<Self, Error> {
        if !source.is_file() {
            return Err(Error::InvalidSourceFile);
        }

        let source_str = source.as_path().display().to_string();
        let output = if let Some(output) = output {
            // if it's a directory, use that directory and the name of the source file
            // for the html name.
            if output.is_dir() {
                let filename = {
                    let filename = source
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .ok_or(Error::InvalidSourceFile)?;
                    format!("{}.html", filename)
                };
                format!("{}/{}", output.as_path().display().to_string(), filename)
            } else {
                output.as_path().display().to_string()
            }
        } else {
            let source = source
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or(Error::InvalidSourceFile)?;
            format!("{}.html", source)
        };

        let (lang, cmnt, extn) = if let Some(ext) = source.extension().and_then(|s| s.to_str()) {
            let lang = LANGUAGES
                .get(ext)
                .ok_or_else(|| Error::UnsupportedExt(ext.to_string()))?;
            (&lang.name, &lang.comment, ext.to_string())
        } else {
            return Err(Error::InvalidSourceFile);
        };

        Ok(Self {
            sections: vec![],
            filename: source_str,
            css: include_str!("assets/template.css").to_string(),
            html: include_str!("assets/template.html").to_string(),
            output,
            language: lang.to_string(),
            extension: extn,
            doc_symbol: cmnt.to_string(),
        })
    }

    pub fn render(&self) -> Result<(), Error> {
        let template =
            Template::new(self.html.as_str()).map_err(|_| Error::InvalidTemplateSource)?;
        let path = std::path::Path::new(&self.output);
        let prefix = path.parent().ok_or(Error::RenderFailed)?;
        std::fs::create_dir_all(prefix)?;
        template.render_to_file(&self.output, self)?;
        Ok(())
    }

    fn parse_code(
        &self,
        iter: &mut Peekable<Lines<BufReader<File>>>,
        code_buffer: &mut String,
    ) -> Result<(), Error> {
        while let Some(Ok(next_line)) = iter.peek() {
            let line_trimmed = next_line.trim_start();
            if !line_trimmed.starts_with(&self.doc_symbol) && !line_trimmed.is_empty() {
                let next_line = next_line.replace("<", "&lt");
                let next_line = next_line.replace(">", "&gt");
                code_buffer.push_str(&next_line);
                if !line_trimmed.ends_with('\n') {
                    code_buffer.push_str("\n");
                }

                iter.next();
            } else {
                return Ok(());
            }
        }

        Ok(())
    }

    fn parse_doc(
        &self,
        iter: &mut Peekable<Lines<BufReader<File>>>,
        doc_buffer: &mut String,
    ) -> Result<(), Error> {
        while let Some(Ok(next_line)) = iter.peek() {
            if next_line.trim().starts_with(&self.doc_symbol) {
                // rust specific doc comments
                if next_line.trim().starts_with("///") {
                    doc_buffer.push_str(next_line.trim_start());
                    doc_buffer.push_str("\n");
                } else {
                    let next_line = next_line.trim_start().trim_start_matches(&self.doc_symbol);
                    doc_buffer.push_str(next_line);
                    doc_buffer.push_str("\n");
                }
                iter.next();
            } else {
                return Ok(());
            }
        }

        Ok(())
    }

    pub fn parse(&mut self) -> Result<(), Error> {
        let fs = BufReader::new(OpenOptions::new().read(true).open(&self.filename)?);
        let mut lines = fs.lines().peekable();
        let mut idx = 0;
        while let Some(Ok(next_line)) = lines.peek() {
            if next_line.is_empty() {
                lines.next();
                continue;
            }
            let mut doc = String::new();
            let mut code = String::new();
            self.parse_doc(&mut lines, &mut doc)?;
            self.parse_code(&mut lines, &mut code)?;
            let docs_html = comrak::markdown_to_html(&doc, &comrak::ComrakOptions::default());
            let section = Section {
                num: idx,
                docs_html,
                code_html: code,
            };
            self.sections.push(section);
            idx += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Docco;
    use std::path::{Path, PathBuf};

    #[test]
    fn render_rust_source() {
        let mut docco = Docco::new(PathBuf::from("tests/samples/source.rs"), None).unwrap();
        docco.parse().unwrap();
        docco.render().unwrap();

        assert_eq!(docco.sections.len(), 23);
        std::fs::remove_file(docco.output).unwrap();
    }

    #[test]
    fn render_ruby_source() {
        let mut docco = Docco::new(PathBuf::from("tests/samples/source.rb"), None).unwrap();
        docco.parse().unwrap();
        docco.render().unwrap();

        assert_eq!(docco.sections.len(), 158);
        std::fs::remove_file(docco.output).unwrap();
    }

    #[test]
    fn render_go_source() {
        let mut docco = Docco::new(PathBuf::from("tests/samples/gocco.go"), None).unwrap();
        docco.parse().unwrap();
        docco.render().unwrap();

        assert_eq!(docco.sections.len(), 56);
        std::fs::remove_file(docco.output).unwrap();
    }

    #[test]
    fn render_python_source() {
        let mut docco = Docco::new(PathBuf::from("tests/samples/source.py"), None).unwrap();
        docco.parse().unwrap();
        docco.render().unwrap();

        assert_eq!(docco.sections.len(), 248);
        std::fs::remove_file(docco.output).unwrap();
    }

    #[test]
    fn uses_source_filename_in_same_output_dir_when_output_is_specified_as_a_directory() {
        let mut docco = Docco::new(
            PathBuf::from("tests/samples/source.rs"),
            Some(PathBuf::from("tests/")),
        )
        .unwrap();
        docco.parse().unwrap();
        docco.render().unwrap();

        assert!(Path::new("tests/source.html").exists());
        std::fs::remove_file(docco.output).unwrap();
    }

    #[test]
    fn should_output_in_current_dir_using_sourcename_when_no_output_specified() {
        let mut docco = Docco::new(PathBuf::from("tests/samples/source.rs"), None).unwrap();
        docco.parse().unwrap();
        docco.render().unwrap();

        assert!(Path::new("source.html").exists());
        std::fs::remove_file("source.html").unwrap();
    }
}
