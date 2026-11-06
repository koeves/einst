//! The response plots of the Parametric EQ, DJ EQ, Vari-EQ and Filter
//! effect editors (their `CtrlPlot`, named `Plot`).
//!
//! v1's editors keep their own copy of the effect, push the effect's
//! parameter values into it whenever they change, and redraw the plot
//! from the node's display curves (`ATC_Tick`: Parametric EQ [re:fd397a],
//! DJ EQ [re:ed3666], Vari-EQ [re:412abb], Filter [re:646302]; the copy
//! runs at the engine's sample rate, [re:859a70]). Each curve is one of
//! the plot's Y vectors in a named colour (`LoadSkin` [re:7b77e0],
//! [re:5c0fae], [re:7b6dc7], [re:ab31ad]), on axes 0..1:
//!
//! - Parametric EQ: the two mid bands (`EQPlotBand0Line`, `EQPlotBand1Line`);
//! - DJ EQ: low, mid and high bands (`EQPlotBand0Line`, `EQPlotBand2Line`,
//!   `EQPlotBand3Line`);
//! - Vari-EQ: its filter (`EQPlotSingleBandLine`);
//! - Filter: the low-pass while its active button is on (`EQPlotBand0Line`),
//!   the high-pass while its active button is on (`EQPlotBand1Line`).
//!
//! A band that is switched off draws the flat 0 dB line (the Filter's
//! switched-off parts are not drawn). The curves come from
//! `einst_dsp::eq_display`; they reach the renderer as the plot's
//! `curves` attribute.

use dreist_formats::Node;
use dreist_gui::Ctrl;
use einst_dsp::eq_display::{POINTS, x_at};

/// The engine's sample rate when the editor has not been told one.
pub const DEFAULT_SAMPLE_RATE: f32 = 44100.0;

/// The effect's front-panel values as the engine pushes them (the base
/// values, `<name>` or `Parameter<k>`, else 0).
fn params(kind: usize, node: &Node) -> Vec<f32> {
    dreist_formats::params::effect_param_names(kind)
        .iter()
        .enumerate()
        .map(|(k, name)| {
            let stripped: String = name.chars().filter(|c| !c.is_whitespace()).collect();
            node.f32(&stripped).or_else(|| node.f32(&format!("Parameter{k}"))).unwrap_or(0.0)
        })
        .collect()
}

/// The node's port values after the hidden ports and `params` are set,
/// as the editor's effect copy holds them.
fn ports(kind: usize, params: &[f32]) -> Option<Vec<dreist_dsp::fx::Port>> {
    let mut node = einst_dsp::create(kind)?;
    let ports = node.ports();
    for &(i, v) in einst_dsp::internal_ports(kind) {
        if let Some(p) = ports.get_mut(i) {
            p.set01(v);
        }
    }
    for (k, v) in params.iter().enumerate() {
        if let Some(p) = ports.get_mut(dreist_dsp::fx::nodes::node_port(kind, k)) {
            p.set01(*v);
        }
    }
    Some(ports.clone())
}

/// The plot's curves (colour name, y per point) for effect `kind` with
/// front-panel values `params`; `buttons` are the Filter's HP / LP
/// active buttons. `None` for effects without a plot.
pub fn curves(kind: usize, params: &[f32], buttons: (bool, bool), sample_rate: f32) -> Option<Vec<(&'static str, [f32; POINTS])>> {
    use einst_dsp::{DJ_EQ, FILTER, PARAMETRIC_EQ, VARI_EQ, ssl_eq, ssl_filter, vari_eq};
    if ![PARAMETRIC_EQ, DJ_EQ, VARI_EQ, FILTER].contains(&kind) {
        return None;
    }
    let ports = ports(kind, params)?;
    let v: Vec<f32> = ports.iter().map(|p| p.value).collect();
    let sr = sample_rate;
    Some(match kind {
        PARAMETRIC_EQ => vec![("EQPlotBand0Line", ssl_eq::display_part(&v, 1, sr)), ("EQPlotBand1Line", ssl_eq::display_part(&v, 2, sr))],
        DJ_EQ => vec![("EQPlotBand0Line", ssl_eq::display_part(&v, 0, sr)), ("EQPlotBand2Line", ssl_eq::display_part(&v, 2, sr)), ("EQPlotBand3Line", ssl_eq::display_part(&v, 3, sr))],
        VARI_EQ => vec![("EQPlotSingleBandLine", vari_eq::display(&ports, sr))],
        _ => {
            let (hp, lp) = buttons;
            let mut c = Vec::new();
            if lp {
                c.push(("EQPlotBand0Line", ssl_filter::display_part(&v, 0, sr)));
            }
            if hp {
                c.push(("EQPlotBand1Line", ssl_filter::display_part(&v, 1, sr)));
            }
            c
        }
    })
}

/// The `curves` attribute for `curves`: `colour|x,y x,y …` joined by `;`.
pub fn curves_attr(curves: &[(&str, [f32; POINTS])]) -> String {
    curves
        .iter()
        .map(|(col, ys)| {
            let pts: Vec<String> = ys.iter().enumerate().map(|(i, y)| format!("{},{}", x_at(i), y)).collect();
            format!("{col}|{}", pts.join(" "))
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// Fill the `Plot` of effect slot panel `slot` (effect `kind`, document
/// node `node`); recomputed only when its inputs change.
pub fn sync(slot: &mut Ctrl, kind: usize, node: &Node, sample_rate: f32) {
    let params = params(kind, node);
    // The Filter's Control1 (HP active) and Control3 (LP active).
    let on = |c: &Ctrl, n: &str| c.find(n).is_some_and(|b| b.on);
    let buttons = (on(slot, "Control1"), on(slot, "Control3"));
    let Some(plot) = slot.find_mut("Plot").filter(|p| p.class == "CtrlPlot") else { return };
    let key = format!("{kind}|{params:?}|{buttons:?}|{sample_rate}");
    if plot.attr("plotKey") == Some(key.as_str()) {
        return;
    }
    let Some(c) = curves(kind, &params, buttons, sample_rate) else { return };
    plot.set_attr("curves", curves_attr(&c));
    plot.set_attr("plotKey", key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plotted_effects_and_their_curves() {
        let flat = |c: &[f32; POINTS]| c.iter().all(|y| (*y - 0.5).abs() < 1e-4);
        assert!(curves(1, &[0.5], (true, true), 44100.0).is_none());
        // Parametric EQ: two mid-band curves, flat at 0 dB gain (0.5).
        let p = curves(3, &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5], (false, false), 44100.0).unwrap();
        assert_eq!(p.iter().map(|c| c.0).collect::<Vec<_>>(), ["EQPlotBand0Line", "EQPlotBand1Line"]);
        assert!(p.iter().all(|c| flat(&c.1)));
        // Low gain up: the first curve rises, the second stays flat.
        let p = curves(3, &[0.5, 0.5, 0.9, 0.5, 0.5, 0.5], (false, false), 44100.0).unwrap();
        assert!(p[0].1.iter().any(|y| *y > 0.7) && flat(&p[1].1));
        // DJ EQ: three bands.
        let d = curves(4, &[0.5, 0.5, 0.5, 0.5], (false, false), 44100.0).unwrap();
        assert_eq!(d.iter().map(|c| c.0).collect::<Vec<_>>(), ["EQPlotBand0Line", "EQPlotBand2Line", "EQPlotBand3Line"]);
        // Vari-EQ: one.
        assert_eq!(curves(5, &[0.0, 0.5, 0.5, 0.5], (false, false), 44100.0).unwrap().len(), 1);
        // Filter: only the parts whose buttons are on (LP first).
        let f = |b| curves(28, &[1.0, 0.3, 1.0, 0.7], b, 44100.0).unwrap().iter().map(|c| c.0).collect::<Vec<_>>();
        assert_eq!(f((true, true)), ["EQPlotBand0Line", "EQPlotBand1Line"]);
        assert_eq!(f((true, false)), ["EQPlotBand1Line"]);
        assert!(f((false, false)).is_empty());
    }

    /// The engine mixer's Parametric EQ and Filter slots get their curves,
    /// and they follow the parameters and the sample rate.
    #[test]
    fn effect_slots_fill_their_plots() {
        use crate::{Editor, front, mixer, model::Model, project};
        crate::app::test_setup();
        let Ok(mut ed) = Editor::new(dreist_gui::skin::default_theme(), Model::new(project::new_document())) else { return };
        ed.model.editor_tab = front::ENGINE_MIXER_PAGE;
        let e = ed.model.engine;
        let chain = ed.model.engine_mut(e).child_mut_or_insert("EffectChain", "EffectChain");
        chain.children.retain(|c| c.class != "Effect");
        let mut eq = dreist_formats::Node::new("Effect", Some("Effect0"));
        eq.set("EffectType", dreist_formats::params::EFFECT_TYPES[3]).set("IsActive", "true");
        for (k, v) in ["LowQ", "Lowfreq", "Lowgain", "HighQ", "Highfreq", "Highgain"].iter().zip(["0.5", "0.5", "0.9", "0.5", "0.5", "0.5"]) {
            eq.set(k, v);
        }
        chain.children.push(eq);
        let mut flt = dreist_formats::Node::new("Effect", Some("Effect1"));
        flt.set("EffectType", dreist_formats::params::EFFECT_TYPES[28]).set("IsActive", "true");
        for (k, v) in ["HPactive", "HPfreq", "LPactive", "LPfreq"].iter().zip(["0", "0.3", "1", "0.7"]) {
            flt.set(k, v);
        }
        chain.children.push(flt);
        ed.sync();
        let page = ed.mixers.iter().find(|m| m.kind == mixer::Kind::Engine).unwrap().page.clone();
        let row = [page.clone(), ed.app.ctrl(&page).unwrap().path_of("ChainPanelScroll").unwrap()].concat();
        let plot = |ed: &Editor, s: usize| ed.app.ctrl(&[row.clone(), vec![s]].concat()).and_then(|c| c.find("Plot")).and_then(|p| p.attr("curves")).map(str::to_string);
        let a = plot(&ed, 0).expect("the EQ plot has curves");
        assert_eq!(a.split(';').map(|c| c.split('|').next().unwrap()).collect::<Vec<_>>(), ["EQPlotBand0Line", "EQPlotBand1Line"]);
        // The Filter: its HP is off, so only the low-pass.
        let f = plot(&ed, 1).expect("the Filter plot has curves");
        assert_eq!(f.split(';').map(|c| c.split('|').next().unwrap()).collect::<Vec<_>>(), ["EQPlotBand0Line"]);
        // Another rate, other curves.
        ed.model.sample_rate = Some(8000.0);
        ed.sync();
        assert_ne!(plot(&ed, 0).unwrap(), a);
    }

    #[test]
    fn the_attribute_lists_256_points_per_curve() {
        let a = curves_attr(&[("A", [0.5; POINTS]), ("B", [0.25; POINTS])]);
        let parts: Vec<&str> = a.split(';').collect();
        assert_eq!(parts.len(), 2);
        let (col, pts) = parts[1].split_once('|').unwrap();
        assert_eq!(col, "B");
        let pts: Vec<&str> = pts.split_whitespace().collect();
        assert_eq!(pts.len(), POINTS);
        assert_eq!(pts[0], "0,0.25");
        assert_eq!(pts[128], "0.5,0.25");
    }
}
