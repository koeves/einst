//! Geist v1 documents.
//!
//! Status per file kind (see `docs/re/formats-v1.md`):
//! * effect presets, chains, pattern and swing templates: same schema as
//!   Geist2 [file]: 198 of 260 factory presets are byte-identical. Loaded as is.
//! * project, kit, pad, engine, pattern, bank: converted onto Geist2's tree by
//!   [`crate::geist2`], following v1's `SerializeAsXML` bodies and Geist2's
//!   legacy load branches (`docs/re/loading-v1.md`) [decomp].

use std::path::Path;

use geist_formats::Node;

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
    Ok(convert_file(path)?.map(|doc| (as_project(kind, doc), geist_formats::model::samples_dir(path))))
}

/// A Geist v1 project, kit, pad, engine, pattern or bank file converted to
/// Geist2's tree as is (the same root shape: one `Pad`, `Engine` or
/// `StepPattern` for single-object files); `Ok(None)` for anything else.
pub fn convert_file(path: &Path) -> Result<Option<Node>, Error> {
    let Some(kind) = FileKind::from_path(path) else { return Ok(None) };
    if matches!(kind, FileKind::EffectPreset | FileKind::EffectChain | FileKind::PatternTemplate | FileKind::SwingTemplate) {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path)?;
    if crate::parse(&text)?.version.is_some() {
        return Ok(None);
    }
    Ok(Some(Document::from_str(kind, &text)?.to_geist2()?))
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

/// v1's effect types in v1's order (`Effect::EffectTypeToString` 0xf47f0,
/// ev_16b78c41): index = v1 `EEffectType`. Effect files store the name,
/// which Geist2 and Dreist number differently (`geist_formats::EFFECT_TYPES`).
pub const EFFECT_TYPES: [&str; 33] = [
    "No Effect", "Gain", "Stereo Width", "Parametric EQ", "DJ EQ", "Vari-EQ", "Channel Compressor", "Bus Compressor", "Noise Gate", "Envelope Shaper", "Delay", "Flanger", "Chorus", "Phaser",
    "Distortion", "Bit Crusher", "Ring Mod", "Frequency Shifter", "Filter", "Modulated Filter", "Comb Filter", "Three Oh! Three Filter", "Hoover Filter", "Grandad Filter", "Little Fatty Filter",
    "Child Genius Filter", "Japan Filter", "Audio Freezer", "Breverb Hall", "Breverb Plate", "Breverb Room", "Breverb Inverse", "Tin Can Verb",
];

/// v1's effect menu categories (`Effect::EffectCategoryToString` 0xf4e00, ev_b6c62234).
pub const EFFECT_CATEGORIES: [&str; 6] = ["Distortion", "Dynamics", "EQ", "Filters", "FX", "Reverbs"];

/// `Effect::GetCategoryForEffect` (0xf4720, ev_6494cf3e) for a v1 type index.
pub fn effect_category(v1_type: usize) -> usize {
    match v1_type {
        1 | 2 | 6..=9 => 1,
        3..=5 => 2,
        14..=16 => 0,
        18..=26 => 3,
        28..=32 => 5,
        _ => 4,
    }
}

/// The effect types v1's menu offers in `category` (`EditorEffectPanel::
/// DoShowEffectTypeMenu`, ev_ad49b2ff: "No Effect", then one submenu per
/// category listing its types in type order).
pub fn effect_menu(category: usize) -> impl Iterator<Item = &'static str> {
    EFFECT_TYPES.iter().enumerate().skip(1).filter(move |(i, _)| effect_category(*i) == category).map(|(_, n)| *n)
}

/// An effect of one of v1's types ([`EFFECT_TYPES`]; all of them exist in
/// Geist2 under the same names).
fn check_effect(e: &Node) -> Result<(), Error> {
    let ty = e.attr("EffectType").ok_or_else(|| Error::Shape("Effect", "no EffectType".into()))?;
    if !EFFECT_TYPES.contains(&ty) {
        return Err(Error::Shape("Effect", format!("unknown EffectType {ty:?}")));
    }
    Ok(())
}
