use std::env;

use anyhow::*;
use fs_extra::{copy_items, dir::CopyOptions};

fn main() -> Result<()> {
    // This tells Cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=game_engine/res/*");

    let out_dir = env::var("OUT_DIR")?;

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;

    println!("{:?}", std::env::current_dir());

    let paths_to_copy = vec!["res/"];
    println!("copying: {paths_to_copy:?} -> {out_dir:?}");
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}
