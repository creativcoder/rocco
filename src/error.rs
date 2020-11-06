use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("Failed rendering template")]
    RenderFailed,
    #[error("Template creation failed")]
    InvalidTemplateSource,
    #[error("Invalid source file")]
    InvalidSourceFile,
    #[error("Language map initialization failed")]
    LangMapInitFailed,
    #[error("Doc parse failed")]
    DocParseFailed,
    #[error("Extension not yet supported")]
    UnsupportedExt(String),
    #[error("Could not find extension of source file or Unsupported source file")]
    NoExtension,
}
