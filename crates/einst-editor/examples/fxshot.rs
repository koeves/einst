//! Render effect panels to PNG: `fxshot <out_dir> <scale> <type>...` puts
//! the given effect types (index in `EFFECT_TYPES`, or `-` for an empty
//! slot) four at a time into the engine chain of the demo project, shows the
//! engine mixer page and saves each slot as `<out_dir>/<type>.png` (199 x 196
//! points at `scale`), plus `<out_dir>/page-<n>.png` for each page of four.

use dreist_gui::tiny_skia::IntRect;

/// The type's `Default.fxpreset` from `FXSHOT_PRESETS` (a folder of
/// `<type name>/Default.fxpreset`), parameters named as the document names them.
fn defaults(kind: usize) -> Option<dreist_formats::Node> {
    let dir = std::path::PathBuf::from(std::env::var_os("FXSHOT_PRESETS")?);
    let path = dir.join(dreist_formats::params::EFFECT_TYPES[kind]).join("Default.fxpreset");
    let root = dreist_formats::Node::parse(&std::fs::read_to_string(path).ok()?).ok()?;
    let mut n = root.children.into_iter().find(|c| c.class == "Effect")?;
    let names = dreist_formats::params::effect_param_names(kind);
    for (k, _) in n.attrs.iter_mut() {
        if let Some(pn) = k.strip_prefix("Parameter").and_then(|i| i.parse::<usize>().ok()).and_then(|i| names.get(i)) {
            *k = pn.chars().filter(|c| !c.is_whitespace()).collect();
        }
    }
    Some(n)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = std::path::PathBuf::from(args.get(1).expect("out dir"));
    let scale: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(2.0);
    let kinds: Vec<Option<usize>> = args[3..].iter().map(|s| s.parse().ok()).collect();
    std::fs::create_dir_all(&out).unwrap();
    for (page, group) in kinds.chunks(4).enumerate() {
        let mut ed = einst_editor::screens::editor().expect("demo content");
        ed.model.editor_tab = 6; // the engine mixer page
        let e = ed.model.engine;
        let chain = ed.model.engine_mut(e).child_mut_or_insert("EffectChain", "EffectChain");
        chain.children.retain(|c| c.class != "Effect");
        for (slot, kind) in group.iter().enumerate() {
            let Some(kind) = kind else { continue };
            let mut fx = defaults(*kind).unwrap_or_else(|| dreist_formats::Node::new("Effect", None));
            fx.name = Some(format!("Effect{slot}"));
            fx.set("EffectType", dreist_formats::params::EFFECT_TYPES[*kind]).set("IsActive", "true");
            chain.children.push(fx);
        }
        ed.sync();
        let img = ed.render(scale);
        img.save_png(out.join(format!("page-{page}.png"))).unwrap();
        for (slot, kind) in group.iter().enumerate() {
            let path = ed.app.root.path_of(&format!("FX{}Panel", slot + 1)).expect("slot");
            let (x, y) = ed.app.origin(&path);
            let r = IntRect::from_xywh((x as f32 * scale) as i32, (y as f32 * scale) as i32, (199.0 * scale) as u32, (196.0 * scale) as u32).unwrap();
            let name = kind.map_or_else(|| "empty".to_string(), |k| k.to_string());
            img.clone_rect(r).unwrap().save_png(out.join(format!("{name}.png"))).unwrap();
        }
    }
}
