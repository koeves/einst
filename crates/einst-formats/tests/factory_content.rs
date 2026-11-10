//! Reads every factory file Einst ships (`assets/factory`: v1's effect
//! presets, effect chains, pattern and swing templates) and, when the
//! whole v1 install is at hand (`EINST_V1_ASSETS`, default
//! `../dreist-re/einst/v1` next to this repository), every XML file and
//! skin of it; those parts skip without it.

use std::path::{Path, PathBuf};

use einst_formats::{FileKind, v1::Document};

fn factory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/factory")
}

/// The v1 install tree, if present.
fn v1() -> Option<PathBuf> {
    let dir = std::env::var_os("EINST_V1_ASSETS").map(PathBuf::from).unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dreist-re/einst/v1"));
    if dir.is_dir() {
        Some(dir)
    } else {
        eprintln!("skipped: no v1 install at {} (EINST_V1_ASSETS)", dir.display());
        None
    }
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() { files(&p, out) } else { out.push(p) }
    }
}

/// Loads and converts every document under `dir`; counts by kind.
fn load_all(dir: &Path) -> std::collections::BTreeMap<String, usize> {
    let mut all = Vec::new();
    files(dir, &mut all);
    let mut counts = std::collections::BTreeMap::new();
    for p in &all {
        let Some(kind) = FileKind::from_path(p) else { continue };
        let doc = Document::load(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        doc.to_v2().unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        *counts.entry(format!("{kind:?}")).or_insert(0) += 1;
    }
    counts
}

fn factory_counts(counts: &std::collections::BTreeMap<String, usize>) -> (usize, usize, usize, usize) {
    let n = |k: &str| counts.get(k).copied().unwrap_or(0);
    (n("EffectPreset"), n("EffectChain"), n("PatternTemplate"), n("SwingTemplate"))
}

#[test]
fn every_factory_file_loads() {
    let counts = load_all(&factory());
    assert_eq!(factory_counts(&counts), (260, 48, 35, 4), "{counts:?}");
}

#[test]
fn every_v1_install_file_loads() {
    let Some(dir) = v1() else { return };
    let counts = load_all(&dir);
    assert_eq!(factory_counts(&counts), (260, 48, 35, 4), "{counts:?}");
}

#[test]
fn v1_skins_parse() {
    let Some(dir) = v1() else { return };
    let mut all = Vec::new();
    files(&dir, &mut all);
    let skins: Vec<_> = all.iter().filter(|p| p.extension().is_some_and(|e| e == "skin")).collect();
    assert!(skins.len() == 102);
    for p in skins {
        let text = std::fs::read_to_string(p).unwrap();
        einst_formats::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    }
}
