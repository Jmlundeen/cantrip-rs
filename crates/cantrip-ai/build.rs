//! The engine commit a shard's header names (`shard::COMMIT`): `git rev-parse HEAD` at build time, or "unknown" outside a
//! git checkout.
//!
//! The script reruns only when the commit can have moved: on a change to HEAD (a checkout, or a commit on a detached
//! HEAD) or to the branch it names, as a loose ref or in `packed-refs`. It never watches the index, which any
//! `git status` rewrites, so it says nothing of uncommitted changes: a "-dirty" mark computed only when HEAD moves would
//! go stale at the first edit after it. Paths come from `git rev-parse --git-path`, which resolves a worktree's refs to
//! the common directory, and only paths that exist are watched, since Cargo reruns a script for a missing one every build.

use std::path::Path;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).current_dir(env!("CARGO_MANIFEST_DIR")).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Watches the file `git rev-parse --git-path path` names, if it exists.
fn watch(path: &str) {
    if let Some(file) = git(&["rev-parse", "--git-path", path])
        && Path::new(&file).is_file()
    {
        let file = if Path::new(&file).is_absolute() { file } else { format!("{}/{file}", env!("CARGO_MANIFEST_DIR")) };
        println!("cargo:rerun-if-changed={file}");
    }
}

fn main() {
    let commit = git(&["rev-parse", "HEAD"]).filter(|h| !h.is_empty()).unwrap_or_else(|| "unknown".to_string());
    watch("HEAD");
    if let Some(branch) = git(&["symbolic-ref", "-q", "HEAD"]) {
        watch(&branch);
        watch("packed-refs");
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=CANTRIP_COMMIT={commit}");
}
