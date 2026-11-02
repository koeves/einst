//! Geist v1 documents.
//!
//! Status per file kind (see `docs/re/formats-v1.md`):
//! * effect presets, chains, pattern and swing templates: same schema as
//!   Geist2 [file]: 198 of 260 factory presets are byte-identical. Loaded as is.
//! * project, kit, pad, engine, pattern, bank: structure known from the save
//!   code [sym]; the mapping onto Geist2's tree needs Geist2's legacy load
//!   path and v1's `SerializeAsXML` bodies [decomp, pending].

use std::path::Path;

use geist_formats::{EFFECT_TYPES, Node};

use crate::{Error, FileKind};

/// A v1 file read into the generic tree, checked against its kind.
#[derive(Debug, Clone)]
pub struct Document {
    pub kind: FileKind,
    pub root: Node,
}

impl Document {
    pub fn load(path: &Path) -> Result<Document, Error> {
        let kind = FileKind::from_path(path).ok_or_else(|| Error::Shape("Geist", path.display().to_string()))?;
        let text = std::fs::read_to_string(path)?;
        Document::from_str(kind, &text)
    }

    pub fn from_str(kind: FileKind, text: &str) -> Result<Document, Error> {
        let root = crate::parse(text)?;
        let doc = Document { kind, root };
        doc.check()?;
        Ok(doc)
    }

    /// The one object element a single-object file holds.
    fn single(&self, class: &'static str) -> Result<&Node, Error> {
        let mut it = self.root.children_of(class);
        match (it.next(), it.next()) {
            (Some(n), None) => Ok(n),
            _ => Err(Error::Shape(class, format!("expected one <{class}> under <root>"))),
        }
    }

    fn check(&self) -> Result<(), Error> {
        match self.kind {
            FileKind::EffectPreset => check_effect(self.single("Effect")?),
            FileKind::EffectChain => {
                for e in self.single("EffectChain")?.children_of("Effect") {
                    check_effect(e)?;
                }
                Ok(())
            }
            FileKind::PatternTemplate => self.single("InsertablePattern").map(|_| ()),
            FileKind::SwingTemplate => self.single("SwingTemplate").map(|_| ()),
            // Structure checks for these land with the decompiled loaders.
            FileKind::Project | FileKind::Kit | FileKind::Pad | FileKind::Engine | FileKind::Pattern | FileKind::PatternBank => Ok(()),
        }
    }

    /// The document in Geist2's vocabulary, ready for Dreist's engine.
    pub fn to_geist2(&self) -> Result<Node, Error> {
        match self.kind {
            FileKind::EffectPreset | FileKind::EffectChain | FileKind::PatternTemplate | FileKind::SwingTemplate => Ok(self.root.clone()),
            FileKind::Project => Err(Error::Unsupported("project (.geist)")),
            FileKind::Kit => Err(Error::Unsupported("kit (.geistkit)")),
            FileKind::Pad => Err(Error::Unsupported("pad (.geistpad)")),
            FileKind::Engine => Err(Error::Unsupported("engine (.geistengine)")),
            FileKind::Pattern => Err(Error::Unsupported("pattern (.geistpattern)")),
            FileKind::PatternBank => Err(Error::Unsupported("pattern bank (.geistbank)")),
        }
    }
}

/// v1's effect types are a subset of Geist2's (`Effect::EffectTypeToString`).
fn check_effect(e: &Node) -> Result<(), Error> {
    let ty = e.attr("EffectType").ok_or_else(|| Error::Shape("Effect", "no EffectType".into()))?;
    if !EFFECT_TYPES.contains(&ty) {
        return Err(Error::Shape("Effect", format!("unknown EffectType {ty:?}")));
    }
    Ok(())
}
