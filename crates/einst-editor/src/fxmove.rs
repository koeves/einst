//! Moving an effect to another slot of its chain (v1's `e_swapFXSlots`,
//! [re:1468df]): the effects between the two slots shift by one
//! ([re:af6f38]), and for a pad's chain every pattern's track of that pad
//! takes its per-slot effect lanes along ([re:d974ef]).

use crate::fxchain::{FxPanel, Target};
use crate::model::Model;

/// [re:af6f38]: the slot each new slot takes its effect from when the one in
/// `from` moves to `to`.
pub(crate) fn layout(from: usize, to: usize) -> [usize; 6] {
    let mut l = [0, 1, 2, 3, 4, 5];
    if from < to && to < 6 {
        for (i, v) in l.iter_mut().enumerate().take(to).skip(from) {
            *v = i + 1;
        }
        l[to] = from;
    } else if to < from && from < 6 {
        for (i, v) in l.iter_mut().enumerate().take(from + 1).skip(to + 1) {
            *v = i - 1;
        }
        l[to] = from;
    }
    l
}

/// Move the effect in slot `from` of `t`'s chain to slot `to`.
pub(crate) fn move_effect(model: &mut Model, t: Target, from: usize, to: usize) -> bool {
    if from == to || from >= 6 || to >= 6 {
        return false;
    }
    let l = layout(from, to);
    let chain = FxPanel::chain_mut(model, t);
    for c in chain.children.iter_mut().filter(|c| c.class == "Effect") {
        let Some(old) = c.name.as_deref().and_then(|n| n.strip_prefix("Effect")).and_then(|n| n.parse::<usize>().ok()) else { continue };
        if let Some(new) = l.iter().position(|&s| s == old) {
            c.name = Some(format!("Effect{new}"));
        }
    }
    // [re:d974ef]: a pad's track keeps its effect lanes with the effects.
    if let Target::Pad(e, p) = t {
        let track = format!("Track{p}");
        for sp in model.engine_mut(e).children.iter_mut().filter(|c| c.class == "StepPattern") {
            for tr in sp.children.iter_mut().filter(|c| c.class == "Track" && c.name.as_deref() == Some(&track)) {
                for st in tr.children.iter_mut().filter(|c| c.class == "Step") {
                    remap_lanes(st, &l);
                }
            }
        }
    }
    true
}

/// Rename a step's `FX<n>…` lane attributes to their slots' new numbers.
fn remap_lanes(st: &mut dreist_formats::Node, l: &[usize; 6]) {
    for (k, _) in st.attrs.iter_mut() {
        let Some(rest) = k.strip_prefix("FX") else { continue };
        let Some(old) = rest.chars().next().and_then(|c| c.to_digit(10)).map(|d| d as usize).filter(|d| (1..=6).contains(d)) else { continue };
        if let Some(new) = l.iter().position(|&s| s == old - 1) {
            *k = format!("FX{}{}", new + 1, &rest[1..]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_shift_the_slots_between() {
        assert_eq!(layout(1, 4), [0, 2, 3, 4, 1, 5]);
        assert_eq!(layout(4, 1), [0, 4, 1, 2, 3, 5]);
        assert_eq!(layout(2, 2), [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn moving_an_effect_takes_its_pad_lanes() {
        let mut model = Model::new(crate::project::new_document());
        let chain = FxPanel::chain_mut(&mut model, Target::Pad(0, 3));
        for (slot, kind) in [(0, "Gain"), (2, "Enhancer")] {
            let mut n = dreist_formats::Node::new("Effect", Some(&format!("Effect{slot}")));
            n.set("EffectType", kind);
            chain.children.push(n);
        }
        let sp = model.engine_mut(0).child_mut_or_insert("StepPattern", "StepPattern0");
        sp.child_mut_or_insert("Track", "Track3").child_mut_or_insert("Step", "S0").set("FX1Mix", 0.25).set("FX3Param2", 0.5);
        sp.child_mut_or_insert("Track", "Track4").child_mut_or_insert("Step", "S0").set("FX1Mix", 0.75);
        assert!(move_effect(&mut model, Target::Pad(0, 3), 0, 2));
        let chain = FxPanel::chain_mut(&mut model, Target::Pad(0, 3));
        let at = |s: usize| chain.children.iter().find(|c| c.name.as_deref() == Some(&format!("Effect{s}"))).and_then(|c| c.attr("EffectType")).map(str::to_string);
        assert_eq!((at(0), at(1), at(2)), (None, Some("Enhancer".into()), Some("Gain".into())));
        let st = |t: &str| model.engine_node(0).and_then(|e| e.child("StepPattern", "StepPattern0")).and_then(|p| p.child("Track", t)).and_then(|t| t.child("Step", "S0")).cloned().unwrap();
        let s3 = st("Track3");
        assert_eq!((s3.attr("FX3Mix"), s3.attr("FX2Param2"), s3.attr("FX1Mix")), (Some("0.25"), Some("0.5"), None));
        assert_eq!(st("Track4").attr("FX1Mix"), Some("0.75"), "other pads keep theirs");
    }
}
