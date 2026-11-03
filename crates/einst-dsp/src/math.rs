//! Helpers the copied Dreist nodes use (Dreist's `geist_dsp::fx::nodes::math`
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
