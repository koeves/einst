//! File types the original v1 registers ([re:8df586] file extension strings) [sym].

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// `.geist`: whole state, [re:b5f535].
    Project,
    /// `.geistkit`: one engine's 16 pads, [re:58c6b0].
    Kit,
    /// `.geistpad`: one pad, [re:bd8f2d].
    Pad,
    /// `.geistengine`: one engine, [re:06276a].
    Engine,
    /// `.geistpattern`: one step pattern, [re:682a5f].
    Pattern,
    /// `.geistbank`: `StepPattern0..N`, [re:6b9684].
    PatternBank,
    /// `.fxpreset`: one `Effect` (same shape as v2).
    EffectPreset,
    /// `.fxchain`: one [re:49ad8e] (same shape as v2).
    EffectChain,
    /// `.patterntemplate`: [re:05588f].
    PatternTemplate,
    /// `.swingtemplate`: [re:9f001e].
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
