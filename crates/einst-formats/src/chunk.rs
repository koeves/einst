//! v1's plugin state as hosts store it.
//!
//! The state is the bytes of a whole-state project file ([re:53d0e3]):
//! saving writes the project to `<temp>/LastHostChunk.<project ext>` and
//! hands its contents to the host; loading writes the host's bytes there
//! and opens that file. The host keeps them in its session, or in
//! - a program or bank file (`CcnK` header, `FPCh` / `FBCh`: the opaque
//!   chunk after it),
//! - an instrument preset property list (the `data` entry, [re:c66219]),
//! - or wrapped once more for a host's newer plugin format (`VstW`, then
//!   the program or bank).
//!
//! [`state_text`] finds the project text in any of these; [`convert`]
//! turns it into the v2 document when it is a v1 project.

use dreist_formats::Node;

use crate::{Error, FileKind};

/// Extensions of the files a host saves the state in (program, bank,
/// instrument preset).
pub const EXTENSIONS: [&str; 3] = ["fxp", "fxb", "aupreset"];

/// Whether `path` is one of [`EXTENSIONS`].
pub fn is_state_file(path: &std::path::Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| EXTENSIONS.iter().any(|x| x.eq_ignore_ascii_case(e)))
}

fn be32(b: &[u8], at: usize) -> Option<usize> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?) as usize)
}

/// The project text inside a host's state bytes, unwrapping the
/// containers above; `None` when there is no XML document inside.
pub fn state_text(bytes: &[u8]) -> Option<String> {
    let mut b = bytes;
    // A newer-format wrapper: `VstW`, its header size, then the header.
    if b.starts_with(b"VstW") {
        let size = be32(b, 4)?;
        b = b.get(8 + size..)?;
    }
    if b.starts_with(b"CcnK") {
        let (size_at, data_at): (usize, usize) = match b.get(8..12)? {
            // Program: magic, size, kind, version, id, plug version,
            // parameter count, name[28], chunk size.
            b"FPCh" => (56, 60),
            // Bank: … program count, 128 reserved bytes, chunk size.
            b"FBCh" => (156, 160),
            _ => return None,
        };
        let size = be32(b, size_at)?;
        b = b.get(data_at..data_at.checked_add(size)?)?;
    } else if b.starts_with(b"bplist") {
        return None;
    } else if let Some(data) = plist_data(b) {
        return state_text(&data);
    }
    let text = std::str::from_utf8(b).ok()?.trim_start_matches('\u{feff}');
    let t = text.trim_start();
    (t.starts_with("<?xml") || t.starts_with("<root")).then(|| text.trim_end_matches('\0').to_string())
}

/// The `data` entry of an XML property list (base64), if `b` is one.
fn plist_data(b: &[u8]) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(b).ok()?;
    if !text.contains("<plist") {
        return None;
    }
    let key = text.find("<key>data</key>")?;
    let rest = &text[key..];
    let start = rest.find("<data>")? + "<data>".len();
    let end = rest[start..].find("</data>")? + start;
    base64(&rest[start..end])
}

fn base64(s: &str) -> Option<Vec<u8>> {
    let val = |c: u8| -> Option<u32> {
        Some(match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32)
    };
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes().filter(|c| !c.is_ascii_whitespace()) {
        if c == b'=' {
            break;
        }
        acc = (acc << 6) | val(c)?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

/// Whether `text` is a v1 project: a `<root>` without the v2 `version`
/// attribute.
pub fn is_v1_project(text: &str) -> bool {
    crate::parse(text).is_ok_and(|n| n.class == "root" && n.version.is_none())
}

/// A host's state bytes as the v2 project document: `Ok(None)` when they
/// hold no v1 project.
pub fn convert(bytes: &[u8]) -> Result<Option<Node>, Error> {
    let Some(text) = state_text(bytes) else { return Ok(None) };
    if !is_v1_project(&text) {
        return Ok(None);
    }
    Ok(Some(crate::v1::Document::from_str(FileKind::Project, &text)?.to_v2()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATE: &str = r#"<?xml version="1.0" ?><root StateVersion="2.1" CurrentEngine="0"><TimeInfo name="TimeInfo" Tempo="98" TSN="3" TSD="4" /></root>"#;

    fn program(chunk: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(b"CcnK");
        b.extend(((52 + chunk.len()) as u32).to_be_bytes());
        b.extend(b"FPCh");
        b.extend(1u32.to_be_bytes());
        b.extend(b"Abcd");
        b.extend(1u32.to_be_bytes());
        b.extend(0u32.to_be_bytes());
        b.extend([0u8; 28]);
        b.extend((chunk.len() as u32).to_be_bytes());
        b.extend(chunk);
        b
    }

    fn bank(chunk: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(b"CcnK");
        b.extend(((152 + chunk.len()) as u32).to_be_bytes());
        b.extend(b"FBCh");
        b.extend(2u32.to_be_bytes());
        b.extend(b"Abcd");
        b.extend(1u32.to_be_bytes());
        b.extend(1u32.to_be_bytes());
        b.extend([0u8; 128]);
        b.extend((chunk.len() as u32).to_be_bytes());
        b.extend(chunk);
        b
    }

    fn b64(data: &[u8]) -> String {
        const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut s = String::new();
        for c in data.chunks(3) {
            let n = c.iter().enumerate().fold(0u32, |a, (i, &x)| a | (x as u32) << (16 - 8 * i));
            for i in 0..4 {
                s.push(if i <= c.len() { T[(n >> (18 - 6 * i)) as usize & 63] as char } else { '=' });
            }
        }
        s
    }

    #[test]
    fn finds_the_project_in_every_container() {
        let raw = STATE.as_bytes();
        let plist = format!("<?xml version=\"1.0\"?><plist version=\"1.0\"><dict><key>data</key>\n<data>\n{}\n</data><key>name</key><string>x</string></dict></plist>", b64(raw));
        let mut wrapped = b"VstW".to_vec();
        wrapped.extend(8u32.to_be_bytes());
        wrapped.extend(1u32.to_be_bytes());
        wrapped.extend(0u32.to_be_bytes());
        wrapped.extend(bank(raw));
        for b in [raw.to_vec(), program(raw), bank(raw), plist.into_bytes(), wrapped] {
            assert_eq!(state_text(&b).as_deref(), Some(STATE));
        }
        assert_eq!(state_text(b"CcnK\0\0\0\0FxCk"), None, "a parameter program has no chunk");
        assert_eq!(state_text(b"{\"fields\":{}}"), None);
    }

    #[test]
    fn converts_a_v1_state_only() {
        let doc = convert(&program(STATE.as_bytes())).unwrap().expect("v1 state");
        assert_eq!(doc.version, Some(3000));
        assert_eq!(doc.child("TimeInfo", "TimeInfo").and_then(|t| t.f32("Tempo")), Some(98.0));
        // A v2 document is not converted; a v1 state below StateVersion 2.0 is refused.
        assert!(convert(br#"<root version="3000" />"#).unwrap().is_none());
        assert!(convert(br#"<root StateVersion="1.0" />"#).is_err());
    }
}
