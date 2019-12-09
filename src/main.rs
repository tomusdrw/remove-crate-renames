use std::path::PathBuf;
use std::{fs, io};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "crate-rename", about = "Remove crate renames from Cargo.toml")]
struct Opt {
    #[structopt(parse(from_os_str))]
    crate_dir: PathBuf,
}

fn main() -> Result<(), String> {
    let opt = Opt::from_args();
    println!("Working on: {:?}", opt.crate_dir);

    let toml = read_and_parse_toml(&opt.crate_dir)
        .map_err(|e| format!("{:?}", e))?;
    let table = toml.as_table()
        .ok_or_else(|| "Invalid toml file".to_string())?;

    if let Some(deps) = table.get("dependencies") {
        parse_and_rename_deps(deps)?;
    }

    if let Some(dev_deps) = table.get("dev-dependencies") {
        parse_and_rename_deps(dev_deps)?;
    }

    Ok(())
}

fn read_and_parse_toml(crate_dir: &PathBuf) -> io::Result<toml::Value> {
    use std::io::Read;

    let cargo_path = crate_dir.join("Cargo.toml");
    println!("Reading Cargo.toml: {:?}", cargo_path);
    let mut file = fs::File::open(cargo_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(toml::from_str(&contents)?)
}

fn parse_and_rename_deps(val: &toml::Value) -> Result<(), String> {
    let deps: Deps = val.clone().try_into()
        .map_err(|e| format!("Unable to parse deps: {:?}", e))?;

    println!("Deps: {:?}", deps);

    let to_rename = deps
        .into_iter()
        .filter_map(|(k, v)| if let Dependency::Detailed(d) = v {
            d.package.map(|p| (k, p))
        } else {
            None
        });

    let cargo_path = "Cargo.toml";
    for (k, v) in to_rename {
        // toml file
        println!(
            "sed -i \"s~{} = ~{} = ~\" {}",
            k, v, cargo_path
        );
        println!(
            "sed -i \"s~package = \\\"{}\\\",~~\" {}",
            v, cargo_path
        );  
        // rust files
        println!(
            "find -name \"*.rs\" ./src | xargs sed -i \"s~{}~{}~\"",
            to_module_name(&k), to_module_name(&v), 
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
