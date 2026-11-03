//! Reads every XML file of the Geist v1 factory install (`assets/v1`).

use std::path::{Path, PathBuf};

use einst_formats::{FileKind, v1::Document};

fn assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/v1")
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() { files(&p, out) } else { out.push(p) }
    }
}

#[test]
fn every_factory_file_loads() {
    let mut all = Vec::new();
    files(&assets(), &mut all);
    let mut counts = std::collections::BTreeMap::new();
    for p in &all {
        let Some(kind) = FileKind::from_path(p) else { continue };
        let doc = Document::load(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        doc.to_v2().unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        *counts.entry(format!("{kind:?}")).or_insert(0) += 1;
    }
    let n = |k: &str| counts.get(k).copied().unwrap_or(0);
    assert_eq!((n("EffectPreset"), n("EffectChain"), n("PatternTemplate"), n("SwingTemplate")), (260, 48, 35, 4), "{counts:?}");
}

#[test]
fn skins_parse() {
    let mut all = Vec::new();
    files(&assets(), &mut all);
    let skins: Vec<_> = all.iter().filter(|p| p.extension().is_some_and(|e| e == "skin")).collect();
    assert!(skins.len() == 102);
    for p in skins {
        let text = std::fs::read_to_string(p).unwrap();
        einst_formats::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    }
}
