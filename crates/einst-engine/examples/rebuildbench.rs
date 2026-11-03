//! Time an editor rebuild: `Geist::from_document` with a warm sample cache
//! (what the plugin's rebuild worker does after a structural edit).
//!
//!     cargo run --release --example rebuildbench -p geist-engine -- <project.geist> [runs]

fn main() {
    let mut args = std::env::args().skip(1);
    let p = std::path::PathBuf::from(args.next().expect("project path"));
    let runs: usize = args.next().and_then(|v| v.parse().ok()).unwrap_or(50);
    let doc = geist_formats::Project::load(&p).expect("project").raw;
    let samples = geist_formats::model::samples_dir(&p);
    let mut cache = geist_engine::audio::SampleCache::default();
    drop(geist_engine::Geist::from_document(&doc, &samples, 44_100.0, Default::default(), &mut cache));
    let t = std::time::Instant::now();
    for _ in 0..runs {
        drop(geist_engine::Geist::from_document(&doc, &samples, 44_100.0, Default::default(), &mut cache));
    }
    println!("Geist::from_document {:.2} ms", t.elapsed().as_secs_f64() * 1e3 / runs as f64);
}
