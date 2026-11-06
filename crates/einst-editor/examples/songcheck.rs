//! Headless song check: `songcheck out_dir project...`.
//!
//! For each project (opened as Einst opens it): the missing samples, every
//! export variant (song / all engines / each engine, stereo mixdown and all
//! assigned outputs, normalised as needed) with peaks, RMS, NaN and length,
//! and a save → reload round trip (document and an 8 s render).

use std::path::{Path, PathBuf};

use dreist_engine::export::{self, Files, Normalize, Options, Source};

fn stats(l: &[f32], r: &[f32]) -> (f32, f32, bool) {
    let peak = l.iter().chain(r).fold(0.0f32, |m, v| m.max(v.abs()));
    let rms = (l.iter().chain(r).map(|v| (*v as f64).powi(2)).sum::<f64>() / (2 * l.len().max(1)) as f64).sqrt() as f32;
    let nan = l.iter().chain(r).any(|v| !v.is_finite());
    (peak, rms, nan)
}

/// A 24-bit stereo WAV as written by the export (data after a 44-byte header).
fn read_wav24(p: &Path) -> (Vec<f32>, Vec<f32>) {
    let b = std::fs::read(p).unwrap_or_default();
    let data = b.windows(4).position(|w| w == b"data").map(|i| &b[i + 8..]).unwrap_or(&[]);
    let s = |c: &[u8]| (((c[0] as i32) << 8 | (c[1] as i32) << 16 | (c[2] as i32) << 24) >> 8) as f32 / 8_388_608.0;
    let (mut l, mut r) = (Vec::new(), Vec::new());
    for f in data.as_chunks::<6>().0 {
        l.push(s(&f[..3]));
        r.push(s(&f[3..]));
    }
    (l, r)
}

fn strip_paths(n: &mut dreist_formats::Node) {
    for (k, v) in &mut n.attrs {
        if k == "AudioFileName" {
            *v = v.rsplit(['/', '\\']).next().unwrap_or("").to_string();
        }
    }
    n.children.iter_mut().for_each(strip_paths);
}

fn render(doc: &dreist_formats::Node, samples: &Path, secs: usize) -> Vec<f32> {
    let mut cache = dreist_engine::audio::SampleCache::default();
    let mut g = dreist_engine::Dreist::from_document(doc, samples, 44100.0, dreist_engine::RenderOptions::default(), &mut cache).expect("build");
    g.start();
    let (mut l, mut r) = (vec![0.0; 512], vec![0.0; 512]);
    let mut out = Vec::new();
    for _ in 0..secs * 44100 / 512 {
        g.process(&mut l, &mut r);
        out.extend_from_slice(&l);
        out.extend_from_slice(&r);
    }
    out
}

fn main() {
    einst::install();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out_dir = PathBuf::from(&args[0]);
    std::fs::create_dir_all(&out_dir).unwrap();
    for f in &args[1..] {
        let p = Path::new(f);
        let stem = p.file_stem().unwrap().to_string_lossy().into_owned();
        let Some((doc, converted)) = einst_editor::app::read_project(p) else {
            println!("{stem}: does not open");
            continue;
        };
        let samples = dreist_formats::model::samples_dir(p);
        let model = einst_editor::Model::new(doc.clone());
        let missing = einst_editor::filefind::missing_files(&model);
        println!("{stem}: opened (converted {converted}), missing samples {}", missing.len());
        let mut variants: Vec<(String, Source, Files)> =
            vec![("Song Mixdown".into(), Source::Song, Files::Mixdown), ("All Mixdown".into(), Source::AllEngines, Files::Mixdown), ("Song".into(), Source::Song, Files::AllOutputs)];
        for e in 0..8 {
            variants.push((format!("Engine{} Mixdown", e + 1), Source::Engine(e), Files::Mixdown));
        }
        for (name, source, files) in variants {
            let opts = Options { source, files, normalize: Normalize::Prompt, tail: 2.0, looped: false, auto_crop: false, render: Default::default() };
            match export::render_export(&doc, &samples, 44100, &opts, &mut |_| true) {
                Ok(r) => {
                    let clipped = r.clipped;
                    let report = r.finish(&out_dir.join(format!("{stem} {name}.wav")), Some(false)).expect("write");
                    let lines: Vec<String> = report
                        .files
                        .iter()
                        .map(|w| {
                            let (l, rr) = read_wav24(w);
                            let (peak, rms, nan) = stats(&l, &rr);
                            let file = w.file_stem().unwrap().to_string_lossy().into_owned();
                            format!("{} {:.1}s peak {peak:.3} rms {rms:.4}{}", file.trim_start_matches(&format!("{stem} {name}")), l.len() as f32 / 44100.0, if nan { " NaN!" } else { "" })
                        })
                        .collect();
                    println!("  {name}: clipped {clipped} | {}", lines.join("; "));
                }
                Err(e) => println!("  {name}: error {e}"),
            }
        }
        // Save → reload.
        let tmp = out_dir.join("roundtrip");
        std::fs::create_dir_all(&tmp).unwrap();
        let saved = tmp.join(format!("{stem}.einst"));
        let (_, errors) = einst_editor::project::save(&doc, Some(&samples), &saved, false).expect("save");
        let (back, _) = einst_editor::app::read_project(&saved).expect("reload");
        let (mut a, mut b) = (doc.clone(), back.clone());
        strip_paths(&mut a);
        strip_paths(&mut b);
        let ra = render(&doc, &samples, 8);
        let rb = render(&back, &dreist_formats::model::samples_dir(&saved), 8);
        let diff = ra.iter().zip(&rb).fold(0.0f32, |m, (x, y)| m.max((x - y).abs()));
        println!("  round trip: doc {} copy errors {} render max diff {diff:.2e}", if a == b { "same" } else { "DIFFERENT" }, errors.len());
    }
}
