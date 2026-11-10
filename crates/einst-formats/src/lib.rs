//! the original v1.1.1.2 file formats. See dreist-re `einst/docs/re/formats-v1.md`.
//!
//! v1 writes its files with [re:0debf8] (TinyXML). The element
//! shape is the one v2 kept for [re:ae4056]: tag = class,
//! `name` = instance key, other attributes = parameters, string leaves as
//! [re:f7fa23]/[re:05b513] children. So [`dreist_formats::Node`] reads them
//! unchanged; what differs is the vocabulary (class and attribute names,
//! per-object `Ver` floats instead of `version` ints) and the semantics.

pub mod bindings;
pub mod chunk;
pub mod kind;
pub mod v1;
pub mod v2;

pub use kind::FileKind;

#[derive(Debug)]
pub enum Error {
    Formats(dreist_formats::Error),
    Io(std::io::Error),
    Shape(&'static str, String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Formats(e) => std::fmt::Display::fmt(e, f),
            Error::Io(e) => write!(f, "io: {e}"),
            Error::Shape(kind, why) => write!(f, "not a v1 {kind} file: {why}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            // Transparent: the inner error's source.
            Error::Formats(e) => std::error::Error::source(e),
            Error::Io(e) => Some(e),
            Error::Shape(..) => None,
        }
    }
}

impl From<dreist_formats::Error> for Error {
    fn from(e: dreist_formats::Error) -> Self {
        Error::Formats(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// Parse any v1 XML file into the generic tree.
pub fn parse(text: &str) -> Result<dreist_formats::Node, Error> {
    Ok(dreist_formats::Node::parse(text)?)
}
