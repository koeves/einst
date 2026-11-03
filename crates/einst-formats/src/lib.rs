//! Geist v1.1.1.2 file formats. See `docs/re/formats-v1.md`.
//!
//! v1 writes its files with `VFoundation::VXMLFile` (TinyXML). The element
//! shape is the one Geist2 kept for `SerializationNode`: tag = class,
//! `name` = instance key, other attributes = parameters, string leaves as
//! `FixedString`/`VString` children. So [`dreist_formats::Node`] reads them
//! unchanged; what differs is the vocabulary (class and attribute names,
//! per-object `Ver` floats instead of `version` ints) and the semantics.

pub mod kind;
pub mod v1;
pub mod v2;

pub use kind::FileKind;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Formats(#[from] dreist_formats::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a Geist v1 {0} file: {1}")]
    Shape(&'static str, String),
}

/// Parse any v1 XML file into the generic tree.
pub fn parse(text: &str) -> Result<dreist_formats::Node, Error> {
    Ok(dreist_formats::Node::parse(text)?)
}
