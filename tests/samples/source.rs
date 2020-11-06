// A literate program combines code and prose (documentation) in one file format.
// The following ascii diagram depicts how ascii generates this html.

//       +-----------------------------------------+
//       | File containing the program description |
//       | peppered with scraps of program code.   |
//       | This is what the programmer works on.   |
//       |          (e.g. source.rs)               |
//       +-----------------------------------------+
//                          |
//                          v
//            o---------------------------o
//            |           Rocco           |
//            o---------------------------o
//                          |
//             +------------+-------------+
//             |                          |
//             v                          v
//    +------------------+   +--------------------------+
//    |       Prose      |   |           Code           |
//    +------------------+   +--------------------------+

// -------------------------------------------------------------------
// The source and prose below is rocco's output on its own source code.

// `Language` represents various attributes of a language used to
// generate and parse code and prose.
#[derive(Debug, Content, serde::Serialize, serde::Deserialize)]
pub struct Language {
    //  of language (e.g, python, rust, go)
    name: String,
    // the delimiter which denotes a comment (//)
    comment: String,
}

// `LANGUAGES` represents a one time initialized json map of languages with their various attributes.
static LANGUAGES: Lazy<HashMap<&'static str, Language>> = Lazy::new(|| {
    let lang_json = include_str!("assets/languages.json");
    serde_json::from_str(lang_json.as_ref()).expect("Language map initialization failed")
});

// A `Section` represents a parsed chunk of code and prose.
#[derive(Content, Debug)]
pub struct Section {
    num: usize,
    docs_html: String,
    code_html: String,
}

// The `Docco` instance contains all items to successfully render a source code.
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
    // Creates a new Docco instance.
    pub fn new(source: &PathBuf, output: Option<PathBuf>) -> Result<Self, Error> {
        if !source.is_file() {
            return Err(Error::DoccoInitFailed);
        }

        let source_str = source.as_path().display().to_string();
        let output = if let Some(output) = output {
            output.as_path().display().to_string()
        } else {
            let a = source_str.find('.').unwrap();
            let slice = &source_str[..a];
            format!("{}.html", slice)
        };

        let (language, comment, extension) = if let Some(ext) = source_str.split(".").last() {
            let language = LANGUAGES
                .get(ext)
                .map(|l| l)
                .ok_or(Error::UnsupportedExt(ext.to_string()))?;
            (&language.name, &language.comment, ext.to_string())
        } else {
            return Err(Error::DoccoInitFailed);
        };

        Ok(Self {
            sections: vec![],
            filename: source_str,
            css: include_str!("assets/template.css").to_string(),
            html: include_str!("assets/template.html").to_string(),
            output,
            language: language.to_string(),
            extension,
            doc_symbol: comment.to_string(),
        })
    }

    // Sets the output html file
    pub fn set_output(&mut self, output: &str) {
        self.output = output.to_string();
    }

    // Renders the parsed sections to an html file.
    pub fn render(&self) -> Result<(), Error> {
        let template =
            Template::new(self.html.as_str()).map_err(|_| Error::InvalidTemplateSource)?;
        let path = std::path::Path::new(&self.output);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        template.render_to_file(&self.output, self)?;
        Ok(())
    }

    // Parses code blocks
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
                if !line_trimmed.ends_with("\n") {
                    code_buffer.push_str("\n");
                }

                iter.next();
            } else {
                return Ok(());
            }
        }

        Ok(())
    }

    // Parses documentation blocks
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

    // High level method that iterates over lines in the given source file
    // and parses code and prose blocks as `Section`s. Additionally, processes the
    // documentation through markdown.
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
