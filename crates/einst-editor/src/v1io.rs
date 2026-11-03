//! Document files as the editor reads them: a Geist v1 file (project, kit,
//! pad, engine, pattern or bank; see `einst_formats::v1`) converted to
//! Geist2's tree, anything else parsed as is.

use std::path::Path;

use geist_formats::Node;

/// The document element of `path`, converted when it is a Geist v1 file.
pub fn read_document(path: &Path) -> Option<Node> {
    match einst_formats::v1::convert_file(path) {
        Ok(Some(doc)) => Some(doc),
        Ok(None) => Node::parse(&std::fs::read_to_string(path).ok()?).ok(),
        Err(_) => None,
    }
}

/// A project to open: a Geist v1 file converted and placed in a project
/// (single-object files in engine 0), else Dreist's reader.
pub fn read_project(path: &Path) -> Option<(Node, bool)> {
    match einst_formats::v1::open_as_project(path) {
        Ok(Some((doc, _))) => Some((doc, true)),
        Ok(None) => geist_formats::Project::load(path).ok().map(|p| (p.raw, false)),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAYER: &str = r#"<Layer name="Layer0" Ver="3.2" AudioFileName="Beat.wav" Envelope0Param3="-1" />"#;

    fn scratch(name: &str, text: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("einst-editor-v1-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(name);
        std::fs::write(&p, text).unwrap();
        p
    }

    #[test]
    fn v1_files_read_converted_and_geist2_files_as_is() {
        let pad = scratch("P.geistpad", &format!(r#"<root><Pad name="Pad" Ver="2" PadMode="Random">{LAYER}</Pad></root>"#));
        let doc = read_document(&pad).unwrap();
        let layer = &doc.children[0].children[0];
        assert_eq!((layer.class.as_str(), layer.version), ("Layer", Some(3300)));
        assert_eq!(layer.f32("AmpEnvHold"), Some(-1.0));
        let g2 = scratch("G.geistpad", r#"<root version="3000"><Pad name="Pad" Ver="2" /></root>"#);
        assert_eq!(read_document(&g2).unwrap().children[0].attr("Ver"), Some("2"), "a Geist2 file is not converted");
        let (proj, converted) = read_project(&pad).unwrap();
        assert!(converted);
        assert_eq!(proj.children[0].children[0].name.as_deref(), Some("Pad0"));
    }

    /// The editor's part loads (`project::load_part`, the kit browser) take v1 files.
    #[test]
    fn editor_part_loads_take_v1_files() {
        let mut model = crate::Model::new(crate::project::new_document());
        let pad = scratch("Q.geistpad", &format!(r#"<root><Pad name="Pad" Ver="2" PadMode="Random">{LAYER}</Pad></root>"#));
        assert!(crate::project::load_part(&mut model.doc, None, crate::project::Part::Pad(1, 3), &pad));
        let p = model.doc.child("Engine", "Engine1").and_then(|e| e.child("Pad", "Pad3")).unwrap();
        assert_eq!(p.attr("PadMode"), Some("Random"));
        assert_eq!(p.children_of("Layer").next().and_then(|l| l.version), Some(3300));
        let kit = scratch("K.geistkit", &format!(r#"<root><Pad name="Pad5" Ver="2">{LAYER}</Pad></root>"#));
        assert!(crate::browser::load_kit_into_doc(&mut model, &kit));
        let e = model.doc.child("Engine", &format!("Engine{}", model.engine)).unwrap();
        assert!(e.child("Pad", "Pad5").and_then(|p| p.children_of("Layer").next()).is_some_and(|l| l.version == Some(3300)));
    }
}
