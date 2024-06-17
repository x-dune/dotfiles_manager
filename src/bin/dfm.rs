use core::panic;
use std::{
    fs::{self, File},
    os::unix,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use handlebars::Handlebars;
use walkdir::WalkDir;

const INPUT_DIR: &str = "home";
const OUTPUT_DIR: &str = "out";
const VALUES_FILE: &str = "values.toml";
const TEMPLATE_EXTENSION: &str = "hbs"; // handlebars template

static HANDLEBARS: OnceLock<Mutex<Handlebars>> = OnceLock::new();

fn main() -> anyhow::Result<()> {
    initialize_global();

    let input_paths = WalkDir::new(INPUT_DIR)
        .into_iter()
        .filter_map(|x| {
            let entry = x.unwrap();
            let path = entry.path();
            let metadata = fs::metadata(path).unwrap();
            if metadata.is_file() {
                Some(path.to_path_buf())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let has_templates = input_paths
        .iter()
        .any(|x| x.extension().and_then(|s| s.to_str()).unwrap_or("") == TEMPLATE_EXTENSION);
    let mut output_paths = vec![];
    if has_templates {
        let values = get_values();
        for path in input_paths {
            let output_path =
                if path.extension().and_then(|s| s.to_str()).unwrap_or("") == TEMPLATE_EXTENSION {
                    render_template_file(&path, &values)?
                } else {
                    copy_raw_file(&path)?
                };
            output_paths.push(output_path);
        }
    } else {
        for path in input_paths {
            let output_path = copy_raw_file(&path)?;
            output_paths.push(output_path);
        }
    }

    for path in &output_paths {
        create_symlink(path);
    }
    Ok(())
}

fn initialize_global() {
    let _ = HANDLEBARS.get_or_init(|| {
        let mut instance = Handlebars::new();
        instance.register_escape_fn(handlebars::no_escape);
        Mutex::new(instance)
    });
}

fn get_values() -> toml::Table {
    let values_path = Path::new(VALUES_FILE);
    if values_path.exists() {
        println!("values.toml found. Using values.toml as templating values");
        let data = fs::read_to_string(values_path).unwrap();
        data.parse::<toml::Table>().unwrap()
    } else {
        panic!("hbs files used, but values.toml not found");
    }
}

fn create_symlink(original_path: &Path) {
    let home_path = home::home_dir().unwrap();

    let prefix = if original_path.starts_with(OUTPUT_DIR) {
        OUTPUT_DIR
    } else {
        INPUT_DIR
    };
    let target_path = home_path.join(original_path.strip_prefix(prefix).unwrap());

    let _ = fs::remove_file(&target_path);

    unix::fs::symlink(
        home_path.join(original_path.canonicalize().unwrap()),
        &target_path,
    )
    .unwrap();

    println!(
        "Symlinked: {} -> {}",
        &original_path.display(),
        &target_path.display()
    );
}

fn render_template_file(path: &Path, data: &toml::Table) -> anyhow::Result<PathBuf> {
    let mut handlebars = HANDLEBARS.get().unwrap().lock().unwrap();
    let clean_path = path.strip_prefix(INPUT_DIR).unwrap();
    let file_stem = path.file_stem().unwrap();
    let next_path = Path::new(OUTPUT_DIR)
        .join(clean_path.parent().unwrap())
        .join(file_stem);
    let path_str = path.to_str().unwrap();
    handlebars.register_template_file(path_str, path_str)?;
    let _ = fs::create_dir_all(next_path.parent().unwrap());
    let mut output_file = File::create(&next_path)?;
    handlebars.render_to_write(path_str, &data, &mut output_file)?;
    println!("Templated: {} -> {}", &path.display(), &next_path.display());
    Ok(next_path.to_path_buf())
}

fn copy_raw_file(path: &Path) -> anyhow::Result<PathBuf> {
    let clean_path = path.strip_prefix(INPUT_DIR).unwrap();
    let next_path = Path::new(OUTPUT_DIR).join(clean_path);
    let _ = fs::create_dir_all(next_path.parent().unwrap());
    let _ = fs::copy(path, &next_path)?;
    Ok(next_path.to_path_buf())
}
