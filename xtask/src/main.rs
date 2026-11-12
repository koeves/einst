use std::path::{Path, PathBuf};

/// `cargo xtask bundle <package> [cargo args]`, run from this workspace's root.
///
/// nih_plug_xtask takes the outermost directory holding a `Cargo.toml` as the
/// workspace root, so in a worktree nested inside another checkout it would
/// build and bundle the outer checkout. This workspace's root is the parent of
/// this crate; other commands go to nih_plug_xtask unchanged.
fn main() -> nih_plug_xtask::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("xtask lives inside the workspace");
    let mut args = std::env::args().skip(1);
    let command = args.next();
    let (Some("bundle"), Some(package)) = (command.as_deref(), args.next()) else {
        return nih_plug_xtask::main();
    };
    let rest: Vec<String> = args.collect();
    std::env::set_current_dir(root)?;
    let target = std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|| root.join("target"));
    nih_plug_xtask::build(std::slice::from_ref(&package), &rest)?;
    nih_plug_xtask::bundle(&target, &package, &rest, false)
}
