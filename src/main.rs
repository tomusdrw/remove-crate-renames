use std::path::{Path, PathBuf};
use std::{fs, io};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "crate-rename", about = "Remove crate renames from Cargo.toml")]
struct Opt {
    #[structopt(parse(from_os_str))]
    crate_dir: PathBuf,
}

const CARGO_TOML: &str = "Cargo.toml";

fn main() -> Result<(), String> {
    let opt = Opt::from_args();
    eprintln!("Working on: {:?}", opt.crate_dir);
    let cargo_path = if opt.crate_dir.ends_with(CARGO_TOML) {
        opt.crate_dir.clone()
    } else {
        opt.crate_dir.join(CARGO_TOML)
    };
    let toml = read_and_parse_toml(&cargo_path)
        .map_err(|e| format!("{:?}", e))?;
    let table = toml.as_table()
        .ok_or_else(|| "Invalid toml file".to_string())?;

    if let Some(deps) = table.get("dependencies") {
        parse_and_rename_deps(&cargo_path, deps)?;
    }

    if let Some(dev_deps) = table.get("dev-dependencies") {
        parse_and_rename_deps(&cargo_path, dev_deps)?;
    }

    Ok(())
}

fn read_and_parse_toml(cargo_path: &Path) -> io::Result<toml::Value> {
    use std::io::Read;
    eprintln!("Reading Cargo.toml: {:?}", cargo_path);
    let mut file = fs::File::open(cargo_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(toml::from_str(&contents)?)
}

fn parse_and_rename_deps(cargo_path: &Path, val: &toml::Value) -> Result<(), String> {
    let deps: Deps = val.clone().try_into()
        .map_err(|e| format!("Unable to parse deps: {:?}", e))?;

    eprintln!("Deps: {:?}", deps);

    let to_rename = deps.0
        .into_iter()
        .filter_map(|(k, v)| if let Dependency::Detailed(d) = v {
            d.package.map(|p| (k, p))
        } else {
            None
        });

    let src_path = cargo_path
        .parent()
        .expect("Parent present, since we have Cargo.toml at the end.")
        .join("src");
    let src_path_display = src_path.display();
    let cargo_path_display = cargo_path.display();

    println!("set -xeu");
    for (k, v) in to_rename {
        // toml file
        println!(
            "sed -i \"s~^{}[ ]*=[ ]*~{} = ~\" {}",
            k, v, cargo_path_display
        );
        println!(
            "sed -i \"s~package[ ]*=[ ]*\\\"{}\\\"[,]*[ ]*~~\" {}",
            v, cargo_path_display
        );
        // std requirement
        println!(
            "sed -i \"s~\\\"{}/std\\\"~\\\"{}/std\\\"~\" {}",
            k, v, cargo_path_display
        );
        // optional crates
        println!(
            "sed -i \"s~\\\"{}\\\",~\\\"{}\\\",~\" {}",
            k, v, cargo_path_display
        );

        // rust files
        println!(
            "find {} -name \"*.rs\" | xargs sed -i \"s~{}::~{}::~g\"",
            src_path_display, to_module_name(&k), to_module_name(&v), 
        );
        println!(
            "find {} -name \"*.rs\" | xargs sed -i \"s~use {}~use {}~g\"",
            src_path_display, to_module_name(&k), to_module_name(&v), 
        );
    }

    Ok(())
}

fn to_module_name(s: &str) -> String {
    s.replace("-", "_")
}

#[derive(Debug, serde::Deserialize)]
struct Deps(std::collections::BTreeMap<String, Dependency>);

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum Dependency {
    Version(String),
    Detailed(DependencyDetailed),
}

#[derive(Debug, serde::Deserialize)]
struct DependencyDetailed {
    pub package: Option<String>,
}
