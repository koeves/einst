//! File types Geist v1 registers (`Grid` file extension strings) [sym].

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// `.geist`: whole state, `Grid::SaveEntireStateToFile`.
    Project,
    /// `.geistkit`: one engine's 16 pads, `Grid::SerializeCurrentKit`.
    Kit,
    /// `.geistpad`: one pad, `Grid::SerializeCurrentPad`.
    Pad,
    /// `.geistengine`: one engine, `Grid::SerializeCurrentEngine`.
    Engine,
    /// `.geistpattern`: one step pattern, `Grid::SerializeCurrentPattern`.
    Pattern,
    /// `.geistbank`: `StepPattern0..N`, `Grid::SerializeCurrentPatternBank`.
    PatternBank,
    /// `.fxpreset`: one `Effect` (same shape as Geist2).
    EffectPreset,
    /// `.fxchain`: one `EffectChain` (same shape as Geist2).
    EffectChain,
    /// `.patterntemplate`: `InsertablePattern`.
    PatternTemplate,
    /// `.swingtemplate`: `SwingTemplate`.
    SwingTemplate,
}

impl FileKind {
    pub fn from_path(p: &Path) -> Option<FileKind> {
        let ext = p.extension()?.to_str()?.to_ascii_lowercase();
        Some(match ext.as_str() {
            "geist" => FileKind::Project,
            "geistkit" => FileKind::Kit,
            "geistpad" => FileKind::Pad,
            "geistengine" => FileKind::Engine,
            "geistpattern" => FileKind::Pattern,
            "geistbank" => FileKind::PatternBank,
            "fxpreset" => FileKind::EffectPreset,
            "fxchain" => FileKind::EffectChain,
            "patterntemplate" => FileKind::PatternTemplate,
            "swingtemplate" => FileKind::SwingTemplate,
            _ => return None,
        })
    }
}
