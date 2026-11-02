//! Geist v1.1.1.2 file formats. See `docs/re/formats-v1.md`.
//!
//! v1 writes its files with `VFoundation::VXMLFile` (TinyXML). The element
//! shape is the one Geist2 kept for `SerializationNode`: tag = class,
//! `name` = instance key, other attributes = parameters, string leaves as
//! `FixedString`/`VString` children. So [`geist_formats::Node`] reads them
//! unchanged; what differs is the vocabulary (class and attribute names,
//! per-object `Ver` floats instead of `version` ints) and the semantics.

pub mod kind;
pub mod v1;

pub use kind::FileKind;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Formats(#[from] geist_formats::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a Geist v1 {0} file: {1}")]
    Shape(&'static str, String),
    #[error("{0}: reading this v1 file type is not implemented yet")]
    Unsupported(&'static str),
}

/// Parse any v1 XML file into the generic tree.
pub fn parse(text: &str) -> Result<geist_formats::Node, Error> {
    Ok(geist_formats::Node::parse(text)?)
}
