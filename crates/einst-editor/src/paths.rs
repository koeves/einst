//! Einst's own resources: Geist v1's factory data (effect presets, effect
//! chains, pattern and swing templates; `assets/geist1/System`, laid out like
//! Geist2's) where Dreist's editor read `geist_formats::paths::skin_dir`.
//!
//! Lookup order, as `geist_formats::paths` does for Dreist:
//! 1. `EINST_RESOURCES`;
//! 2. `Contents/Resources/Einst` of the bundle this code was loaded from
//!    (`Einst.app`, `Einst.vst3`, `Einst.clap`; found from our own image
//!    path, since a plugin's process is the host), or `Resources/Einst` next
//!    to a bare Windows binary;
//! 3. the repository's `assets/geist1/System`.
//!
//! Dreist's UI graphics (fonts, cursors, skin, metronomes) are still found
//! by `geist-gui` / `geist-engine` through `geist_formats::paths::ui_dir`.

use std::path::PathBuf;

/// Name of Einst's folder inside a bundle's `Contents/Resources`.
pub const BUNDLE_DIR: &str = "Einst";

/// The factory data folder (`Effects/`, `Effect Chains/`, `Pattern
/// Templates/`, `Swing Templates/`).
pub fn resources_dir() -> PathBuf {
    if let Ok(p) = std::env::var("EINST_RESOURCES") {
        return PathBuf::from(p);
    }
    if let Some(p) = bundled_dir(BUNDLE_DIR) {
        return p;
    }
    repo_assets()
}

/// Where the editor's callers expect the skins (`Resources/` inside
/// [`resources_dir`]); they take its parent for the factory data, as with
/// `geist_formats::paths::skin_dir`.
pub fn skin_dir() -> PathBuf {
    resources_dir().join("Resources")
}

/// `assets/geist1/System` in the source tree.
pub fn repo_assets() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/geist1/System"))
}

/// `<bundle>/Contents/Resources/<name>` of the bundle holding this image.
fn bundled_dir(name: &str) -> Option<PathBuf> {
    let image = own_image()?;
    // <bundle>/Contents/MacOS/<binary>
    let bundle = image.parent()?.parent().map(|c| c.join("Resources").join(name));
    // Windows `.exe` and `.clap` are bare files: `Resources/` sits next to them.
    let beside = image.parent().map(|d| d.join("Resources").join(name));
    bundle.into_iter().chain(beside).find(|p| p.is_dir())
}

#[cfg(unix)]
fn own_image() -> Option<PathBuf> {
    #[repr(C)]
    struct DlInfo {
        dli_fname: *const std::os::raw::c_char,
        dli_fbase: *mut std::ffi::c_void,
        dli_sname: *const std::os::raw::c_char,
        dli_saddr: *mut std::ffi::c_void,
    }
    unsafe extern "C" {
        fn dladdr(addr: *const std::ffi::c_void, info: *mut DlInfo) -> std::os::raw::c_int;
    }
    let mut info = DlInfo { dli_fname: std::ptr::null(), dli_fbase: std::ptr::null_mut(), dli_sname: std::ptr::null(), dli_saddr: std::ptr::null_mut() };
    let addr = own_image as *const std::ffi::c_void;
    // SAFETY: dladdr only reads the address and fills `info`.
    let ok = unsafe { dladdr(addr, &mut info) };
    if ok == 0 || info.dli_fname.is_null() {
        return std::env::current_exe().ok();
    }
    // SAFETY: dli_fname is a NUL-terminated path owned by the loader.
    let s = unsafe { std::ffi::CStr::from_ptr(info.dli_fname) };
    Some(PathBuf::from(s.to_string_lossy().into_owned()))
}

#[cfg(not(unix))]
fn own_image() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn repo_assets_hold_the_v1_factory_data() {
        let r = super::repo_assets();
        assert!(r.join("Effects/Gain").is_dir());
        assert!(r.join("Swing Templates").is_dir());
    }
}
