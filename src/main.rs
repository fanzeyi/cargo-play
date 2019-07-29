mod cargo;
mod errors;
mod opt;

use log::debug;
use pathdiff::diff_paths;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::iter::Iterator;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::vec::Vec;

use crate::cargo::CargoManifest;
use crate::errors::CargoPlayError;
use crate::opt::{Opt, RustEdition};

fn parse_inputs(inputs: &Vec<PathBuf>) -> Result<Vec<String>, CargoPlayError> {
    inputs
        .into_iter()
        .map(File::open)
        .map(|res| match res {
            Ok(mut fp) => {
                let mut buf = String::new();
                fp.read_to_string(&mut buf)?;
                Ok(buf)
            }
            Err(e) => Err(CargoPlayError::from(e)),
        })
        .collect()
}

fn extract_headers(files: &Vec<String>) -> Vec<String> {
    files
        .iter()
        .map(|file: &String| -> Vec<String> {
            file.lines()
                .skip_while(|line| line.starts_with("#!") || line.is_empty())
                .take_while(|line| line.starts_with("//#"))
                .map(|line| line[3..].trim_start().into())
                .filter(|s: &String| !s.is_empty())
                .collect()
        })
        .flatten()
        .collect()
}

fn temp_dir(name: PathBuf) -> PathBuf {
    let mut temp = PathBuf::new();
    temp.push(env::temp_dir());
    temp.push(name);
    temp
}

fn rmtemp(temp: &PathBuf) -> Result<(), CargoPlayError> {
    debug!("Cleaning temporary folder at: {:?}", temp);
    std::fs::remove_dir_all(temp)?;
    Ok(())
}

fn mktemp(temp: &PathBuf) {
    debug!("Creating temporary building folder at: {:?}", temp);
    if let Err(_) = std::fs::create_dir(temp) {
        debug!("Temporary directory already exists.");
    }
}

fn write_cargo_toml(
    dir: &PathBuf,
    name: String,
    dependencies: Vec<String>,
    edition: RustEdition,
) -> Result<(), CargoPlayError> {
    let manifest = CargoManifest::new(name, dependencies, edition)?;
    let mut cargo = File::create(dir.join("Cargo.toml"))?;

    cargo.write_all(&toml::to_vec(&manifest).map_err(CargoPlayError::from_serde)?)?;

    Ok(())
}

/// Copy all the passed in sources to the temporary directory. The first in the list will be
/// treated as main.rs.
fn copy_sources(temp: &PathBuf, sources: &Vec<PathBuf>) -> Result<(), CargoPlayError> {
    let destination = temp.join("src");
    std::fs::create_dir_all(&destination)?;

    let mut files = sources.iter();
    let base = if let Some(first) = files.next() {
        let dst = destination.join("main.rs");
        debug!("Copying {:?} => {:?}", first, dst);
        std::fs::copy(first, dst)?;
        first.parent()
    } else {
        None
    };

    if let Some(base) = base {
        files
            .map(|file| -> Result<(), CargoPlayError> {
                let part =
                    diff_paths(file, base).ok_or(CargoPlayError::DiffPathError(file.to_owned()))?;
                let dst = destination.join(part);

                // ensure the parent folder all exists
                if let Some(parent) = dst.parent() {
                    let _ = std::fs::create_dir_all(&parent);
                }

                debug!("Copying {:?} => {:?}", file, dst);
                std::fs::copy(file, dst).map(|_| ()).map_err(From::from)
            })
            .collect::<Result<Vec<_>, _>>()?;
    }

    Ok(())
}

fn run_cargo_build(
    toolchain: Option<String>,
    project: &PathBuf,
    release: bool,
    cargo_option: &str,
) -> Result<ExitStatus, CargoPlayError> {
    let mut cargo = Command::new("cargo");

    if let Some(toolchain) = toolchain {
        cargo.arg(format!("+{}", toolchain));
    }

    let cargo = cargo
        .arg("run")
        .arg("--manifest-path")
        .arg(project.join("Cargo.toml"))
        // FIXME: proper escaping
        .args(cargo_option.split_ascii_whitespace());

    if release {
        cargo.arg("--release");
    }

    cargo
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .status()
        .map_err(From::from)
}

fn main() -> Result<(), CargoPlayError> {
    let args = std::env::args().collect::<Vec<_>>();
    let opt = Opt::parse(args);
    if opt.is_err() {
        return Ok(());
    }
    let opt = opt.unwrap();

    let src_hash = opt.src_hash();
    let temp = temp_dir(opt.temp_dirname());

    if opt.cached && temp.exists() {
        let mut bin_path = temp.join("target");
        if opt.release {
            bin_path.push("release");
        } else {
            bin_path.push("debug");
        }
        // TODO reuse logic to formulate package name, i.e. to_lowercase
        bin_path.push(&src_hash.to_lowercase());
        if bin_path.exists() {
            let mut cmd = Command::new(bin_path);
            return cmd
                .args(opt.args)
                .stderr(Stdio::inherit())
                .stdout(Stdio::inherit())
                .status()
                .map(|_| ())
                .map_err(CargoPlayError::from);
        }
    }

    let files = parse_inputs(&opt.src)?;
    let dependencies = extract_headers(&files);

    if opt.clean {
        rmtemp(&temp)?;
    }
    mktemp(&temp);
    write_cargo_toml(&temp, src_hash.clone(), dependencies, opt.edition)?;
    copy_sources(&temp, &opt.src)?;

    match run_cargo_build(opt.toolchain, &temp, opt.release, &opt.cargo_option)?.code() {
        Some(code) => std::process::exit(code),
        None => std::process::exit(-1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_headers() {
        let inputs = vec![
            r#"//# line 1
//# line 2
// line 3
//# line 4"#,
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        let result = dbg!(extract_headers(&inputs));

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], String::from("line 1"));
        assert_eq!(result[1], String::from("line 2"));
    }
}
