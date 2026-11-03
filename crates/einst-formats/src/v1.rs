//! Geist v1 documents.
//!
//! Status per file kind (see `docs/re/formats-v1.md`):
//! * effect presets, chains, pattern and swing templates: same schema as
//!   Geist2 [file]: 198 of 260 factory presets are byte-identical. Loaded as is.
//! * project, kit, pad, engine, pattern, bank: converted onto Geist2's tree by
//!   [`crate::geist2`], following v1's `SerializeAsXML` bodies and Geist2's
//!   legacy load branches (`docs/re/loading-v1.md`) [decomp].

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
            // `Grid::LoadEntireState` loads nothing when `StateVersion` < 2.0 (missing reads as 0).
            FileKind::Project => match self.root.f32("StateVersion") {
                Some(v) if v >= 2.0 => Ok(()),
                v => Err(Error::Shape("project", format!("StateVersion {v:?} < 2.0"))),
            },
            FileKind::Kit => match self.root.children_of("Pad").next() {
                Some(_) => Ok(()),
                None => Err(Error::Shape("kit", "no <Pad> under <root>".into())),
            },
            FileKind::Pad => self.single("Pad").map(|_| ()),
            FileKind::Engine => self.single("Engine").map(|_| ()),
            FileKind::Pattern => self.single("StepPattern").map(|_| ()),
            FileKind::PatternBank => match self.root.children_of("StepPattern").next() {
                Some(_) => Ok(()),
                None => Err(Error::Shape("pattern bank", "no <StepPattern> under <root>".into())),
            },
        }
    }

    /// The document in Geist2's vocabulary, ready for Dreist's engine.
    pub fn to_geist2(&self) -> Result<Node, Error> {
        match self.kind {
            FileKind::EffectPreset | FileKind::EffectChain | FileKind::PatternTemplate | FileKind::SwingTemplate => Ok(self.root.clone()),
            FileKind::Project | FileKind::Kit | FileKind::Pad | FileKind::Engine | FileKind::Pattern | FileKind::PatternBank => Ok(crate::geist2::convert_root(&self.root)),
        }
    }
}

/// Open a file as a Geist2 project if it is a Geist v1 document: `Ok(None)`
/// for anything else (a Geist2 file of the same extension loads as is).
/// v1 roots carry no `version` attribute (Geist2's always do). Kits, pads,
/// engines, patterns and banks are placed in engine 0 of an otherwise empty
/// project. The second value is the folder relative sample paths resolve in
/// (`<file>.samples`, as both versions do).
pub fn open_as_project(path: &Path) -> Result<Option<(Node, std::path::PathBuf)>, Error> {
    let Some(kind) = FileKind::from_path(path) else { return Ok(None) };
    if matches!(kind, FileKind::EffectPreset | FileKind::EffectChain | FileKind::PatternTemplate | FileKind::SwingTemplate) {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path)?;
    if crate::parse(&text)?.version.is_some() {
        return Ok(None);
    }
    let doc = Document::from_str(kind, &text)?;
    Ok(Some((as_project(kind, doc.to_geist2()?), geist_formats::model::samples_dir(path))))
}

/// A converted single-object document placed in engine 0 of a project.
pub fn as_project(kind: FileKind, converted: Node) -> Node {
    if kind == FileKind::Project {
        return converted;
    }
    let rename = |mut n: Node, name: &str| {
        n.name = Some(name.into());
        n
    };
    let mut engine = Node::new("Engine", Some("Engine0"));
    let mut root = Node::new("root", None);
    root.version = converted.version;
    for c in converted.children {
        match (kind, c.class.as_str()) {
            (FileKind::Engine, "Engine") => engine = rename(c, "Engine0"),
            (FileKind::Pad, "Pad") => engine.children.push(rename(c, "Pad0")),
            (FileKind::Pattern, "StepPattern") => engine.children.push(rename(c, "StepPattern0")),
            (FileKind::Kit | FileKind::PatternBank, _) => engine.children.push(c),
            _ => {}
        }
    }
    root.children.push(engine);
    root
}

/// v1's effect types are a subset of Geist2's (`Effect::EffectTypeToString`).
fn check_effect(e: &Node) -> Result<(), Error> {
    let ty = e.attr("EffectType").ok_or_else(|| Error::Shape("Effect", "no EffectType".into()))?;
    if !EFFECT_TYPES.contains(&ty) {
        return Err(Error::Shape("Effect", format!("unknown EffectType {ty:?}")));
    }
    Ok(())
}
