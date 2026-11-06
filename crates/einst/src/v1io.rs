//! Document files as Einst reads them (`einst_editor::app`'s
//! `read_document` / `read_project`): a v1 file (project, kit, pad, engine, pattern or bank;
//! see `einst_formats::v1`) converted to the v2 tree, anything else parsed
//! as is.

use std::path::Path;

use dreist_formats::Node;

/// The document element of `path`, converted when it is a v1 file.
pub fn read_document(path: &Path) -> Option<Node> {
    match einst_formats::v1::convert_file(path) {
        Ok(Some(doc)) => Some(doc),
        Ok(None) => Node::parse(&std::fs::read_to_string(path).ok()?).ok(),
        Err(_) => None,
    }
}

/// A project to open: a v1 file converted and placed in a project
/// (single-object files in engine 0), else the stored project.
pub fn read_project(path: &Path) -> Option<(Node, bool)> {
    match einst_formats::v1::open_as_project(path) {
        Ok(Some((doc, _))) => Some((doc, true)),
        Ok(None) => dreist_formats::Project::load(path).ok().map(|p| (p.raw, false)),
        Err(_) => None,
    }
}

/// The document of a host session: v1's plugin state (a v1 project text,
/// possibly in a program / bank / preset container; `einst_formats::chunk`)
/// converted, anything else parsed as is.
pub fn session_document(state: &str) -> Option<Node> {
    match einst_formats::chunk::convert(state.as_bytes()) {
        Ok(Some(doc)) => Some(doc),
        Ok(None) => Node::parse(state).ok(),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAYER: &str = r#"<Layer name="Layer0" Ver="3.2" AudioFileName="Beat.wav" Envelope0Param3="-1" />"#;

    fn scratch(name: &str, text: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("einst-v1io-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(name);
        std::fs::write(&p, text).unwrap();
        p
    }

    #[test]
    fn v1_files_read_converted_and_v2_files_as_is() {
        let pad = scratch("P.geistpad", &format!(r#"<root><Pad name="Pad" Ver="2" PadMode="Random">{LAYER}</Pad></root>"#));
        let doc = read_document(&pad).unwrap();
        let layer = &doc.children[0].children[0];
        assert_eq!((layer.class.as_str(), layer.version), ("Layer", Some(3300)));
        assert_eq!(layer.f32("AmpEnvHold"), Some(-1.0));
        let g2 = scratch("G.geistpad", r#"<root version="3000"><Pad name="Pad" Ver="2" /></root>"#);
        assert_eq!(read_document(&g2).unwrap().children[0].attr("Ver"), Some("2"), "a v2 file is not converted");
        let (proj, converted) = read_project(&pad).unwrap();
        assert!(converted);
        assert_eq!(proj.children[0].children[0].name.as_deref(), Some("Pad0"));
    }

    #[test]
    fn v1_session_state_converts() {
        let v1 = r#"<root StateVersion="2.1"><TimeInfo name="TimeInfo" Tempo="90" /></root>"#;
        assert_eq!(session_document(v1).unwrap().version, Some(3000));
        let v2 = r#"<root version="3000" CurrentEngine="1" />"#;
        assert_eq!(session_document(v2).unwrap().attr("CurrentEngine"), Some("1"));
        // A host preset file with the state opens as a project.
        let p = scratch("Song.aupreset", &format!("<?xml version=\"1.0\"?><plist><dict><key>data</key><data>{}</data></dict></plist>", "PHJvb3QgU3RhdGVWZXJzaW9uPSIyLjEiIC8+"));
        let (doc, converted) = read_project(&p).unwrap();
        assert!(converted && doc.version == Some(3000));
    }
}
