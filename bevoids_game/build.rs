use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("include_all_assets.rs");

    let mut file = File::create(&dest_path).unwrap();
    file.write_all("pub fn include_all_assets(in_memory: &mut crate::asset_io::InMemoryAssetIo){\n".as_ref())
        .unwrap();

    let dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("assets");
    eprintln!("{:?}", dir);

    visit_dirs(&dir)
        .iter()
        .filter(|path| {
            if path
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .map(|ext| [
                    "png",
                    "jpg",
                    "wav",
                    "mp3",
                    "ttf",
                    "names"].contains(&ext))
                .unwrap_or_default()
            {
                true
            } else {
                cargo_emit::warning!("Unmanaged file: {}", path.to_string_lossy());
                false
            }
        })
        .map(|path| (path, path.strip_prefix(&dir).unwrap()))
        .for_each(|(fullpath, path)| {
            file.write_all(
                format!(
                    r#"in_memory.add_entity(std::path::Path::new({:?}), include_bytes!({:?}));
"#,
                    path.to_string_lossy(),
                    fullpath.to_string_lossy()
                )
                .as_ref(),
            )
            .unwrap();
        });

    file.write_all("}".as_ref()).unwrap();

    cargo_emit::rerun_if_changed!("assets");
}

fn visit_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut collected = vec![];
    if dir.is_dir() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                collected.append(&mut visit_dirs(&path));
            } else {
                collected.push(path);
            }
        }
    }
    collected
}
