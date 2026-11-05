//! Helpers the copied Dreist nodes use (Dreist's `dreist_dsp::fx::nodes::math`
//! is private to its crate; these are copied unchanged).

/// Clamp to 0..1 the way the binaries do: `((|x| + 1) − |x − 1|)/2`.
#[inline]
pub(crate) fn clip01(x: f32) -> f32 {
    ((x.abs() + 1.0) - (x - 1.0).abs()) * 0.5
}

/// Soft saturator: clamp ±2 then `((|c|/16 − 1/4)c² + 1)c`.
#[inline]
pub(crate) fn sat(x: f32) -> f32 {
    let c = ((x + 2.0).abs() - (x - 2.0).abs()) * 0.5;
    ((c.abs() * 0.0625 - 0.25) * c * c + 1.0) * c
}

/// 4-point cubic read at `pos` in a power-of-two ring, in v1's order: the
/// constant term `b·−0.5 + c + a·−1/3 + d·−1/6` left to right (v1's
/// Delay ev_0cdaf927, Comb ev_8625fc5a, Audio Freezer ev_544675a5, Flanger
/// ev_0503858b; Geist2 groups it `((c − a/3) − b/2) − d/6`).
#[inline]
pub(crate) fn cubic(buf: &[f32], mask: u32, pos: f32) -> f32 {
    let i = pos.floor() as i32 as u32;
    let t = pos - (i as i32) as f32;
    let a = buf[(i.wrapping_sub(1) & mask) as usize];
    let b = buf[(i & mask) as usize];
    let c = buf[(i.wrapping_add(1) & mask) as usize];
    let d = buf[(i.wrapping_add(2) & mask) as usize];
    ((((b - c) * 0.5 + (d - a) * 0.16666667) * t + ((a + c) * 0.5 - b)) * t + (b * -0.5 + c + a * -0.33333334 + d * -0.16666667)) * t + b
}

/// Ring sizes: the smallest power of two ≥ `n` (1 below 2).
pub(crate) fn pow2_at_least(n: i64) -> usize {
    if n > 1 { (n as usize).next_power_of_two() } else { 1 }
}
