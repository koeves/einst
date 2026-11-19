use std::path::{Path, PathBuf};

/// `cargo xtask bundle <package> [cargo args]` and `bundle-universal` (a
/// macOS universal binary, Apple Silicon and Intel), run from this
/// workspace's root.
///
/// nih_plug_xtask takes the outermost directory holding a `Cargo.toml` as the
/// workspace root, so in a worktree nested inside another checkout it would
/// build and bundle the outer checkout. This workspace's root is the parent of
/// this crate; other commands go to nih_plug_xtask unchanged.
fn main() -> nih_plug_xtask::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("xtask lives inside the workspace");
    let mut args = std::env::args().skip(1);
    let command = args.next();
    let universal = match command.as_deref() {
        Some("bundle") => false,
        Some("bundle-universal") => true,
        _ => return nih_plug_xtask::main(),
    };
    let Some(package) = args.next() else {
        return nih_plug_xtask::main();
    };
    let rest: Vec<String> = args.collect();
    std::env::set_current_dir(root)?;
    let target = std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|| root.join("target"));
    let package = std::slice::from_ref(&package);
    if universal {
        for triple in ["x86_64-apple-darwin", "aarch64-apple-darwin"] {
            nih_plug_xtask::build(package, &[rest.as_slice(), &[format!("--target={triple}")]].concat())?;
        }
    } else {
        nih_plug_xtask::build(package, &rest)?;
    }
    nih_plug_xtask::bundle(&target, &package[0], &rest, universal)
}
