use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::vec::Vec;

use log::debug;
use pathdiff::diff_paths;

use crate::cargo::CargoManifest;
use crate::errors::CargoPlayError;
use crate::options::{Options, RustEdition};

pub fn read_stdin() -> Result<String, CargoPlayError> {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

pub fn read_files(inputs: &[PathBuf]) -> Result<Vec<(String, &Path)>, CargoPlayError> {
    inputs
        .iter()
        .map(AsRef::as_ref)
        .map(|p: &Path| {
            let mut fp = File::open(p)?;
            let mut buf = String::new();
            fp.read_to_string(&mut buf)?;
            Ok((buf, p))
        })
        .collect()
}

pub fn extract_headers(stdin: Option<&str>, sources: &[&str]) -> Vec<String> {
    stdin
        .iter()
        .chain(sources.iter())
        .map(|source| -> Vec<String> {
            source
                .lines()
                .skip_while(|line| line.starts_with("#!") || line.is_empty())
                .take_while(|line| line.starts_with("//#"))
                .map(|line| line[3..].trim_start().into())
                .filter(|s: &String| !s.is_empty())
                .collect()
        })
        .flatten()
        .collect()
}

pub fn temp_dir(name: PathBuf) -> PathBuf {
    let mut temp = PathBuf::new();
    temp.push(env::temp_dir());
    temp.push(name);
    temp
}

/// This function ignores the error intentionally.
pub fn rmtemp(temp: &PathBuf) {
    debug!("Cleaning temporary folder at: {:?}", temp);
    let _ = std::fs::remove_dir_all(temp);
}

pub fn mktemp(temp: &PathBuf) {
    debug!("Creating temporary building folder at: {:?}", temp);
    if std::fs::create_dir(temp).is_err() {
        debug!("Temporary directory already exists.");
    }
}

pub fn write_cargo_toml(
    dir: &PathBuf,
    name: String,
    dependencies: Vec<String>,
    edition: RustEdition,
    infers: HashSet<String>,
) -> Result<(), CargoPlayError> {
    let mut manifest = CargoManifest::new(name, dependencies, edition)?;
    let mut cargo = File::create(dir.join("Cargo.toml"))?;

    manifest.add_infers(infers);

    cargo.write_all(&toml::to_vec(&manifest).map_err(CargoPlayError::from_serde)?)?;

    Ok(())
}

/// Copy all the passed in sources to the temporary directory. The first in the list will be
/// treated as main.rs.
pub fn copy_sources(
    temp: &PathBuf,
    stdin: Option<&str>,
    files: &[(String, &Path)],
) -> Result<(), CargoPlayError> {
    let destination = temp.join("src");
    std::fs::create_dir_all(&destination)?;

    let mut files = files.iter();
    let base: Option<PathBuf> = if let Some(main) = stdin {
        let dst = destination.join("main.rs");
        debug!("Copying stdin => {:?}", dst);
        std::fs::write(dst, main)?;
        Some(std::env::current_dir()?)
    } else if let Some((main, first)) = files.next() {
        let dst = destination.join("main.rs");
        debug!("Copying {:?} => {:?}", first, dst);
        std::fs::write(dst, main)?;
        first.parent().map(|p| p.to_path_buf())
    } else {
        None
    };

    if let Some(base) = &base {
        files
            .map(|(source, file)| -> Result<(), CargoPlayError> {
                let part = diff_paths(file, base)
                    .ok_or_else(|| CargoPlayError::DiffPathError(file.to_path_buf()))?;
                let dst = destination.join(part);

                // ensure the parent folder all exists
                if let Some(parent) = dst.parent() {
                    let _ = std::fs::create_dir_all(&parent);
                }

                debug!("Copying {:?} => {:?}", file, dst);
                std::fs::write(dst, source)?;
                Ok(())
            })
            .collect::<Result<Vec<_>, _>>()?;
    }

    Ok(())
}

pub fn run_cargo_build(options: &Options, project: &PathBuf) -> Result<ExitStatus, CargoPlayError> {
    let mut cargo = Command::new("cargo");

    if let Some(toolchain) = options.toolchain.as_ref() {
        cargo.arg(format!("+{}", toolchain));
    }

    let subcommand = if options.test {
        "test"
    } else if options.check {
        "check"
    } else if let Some(mode) = options.mode.as_ref() {
        mode.as_str()
    } else {
        "run"
    };

    cargo
        .arg(subcommand)
        .arg("--manifest-path")
        .arg(project.join("Cargo.toml"));

    if let Some(cargo_option) = options.cargo_option.as_ref() {
        // FIXME: proper escaping
        cargo.args(cargo_option.split_ascii_whitespace());
    }

    if options.release {
        cargo.arg("--release");
    }

    if options.quiet {
        cargo.arg("--quiet");
    }

    if options.verbose != 0 {
        for _ in 0..options.verbose {
            cargo.arg("-v");
        }
    }

    cargo
        .arg("--")
        .args(options.args.clone())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .status()
        .map_err(From::from)
}

pub fn copy_project<T: AsRef<Path>, U: AsRef<Path>>(
    from: T,
    to: U,
) -> Result<ExitStatus, CargoPlayError> {
    let to = to.as_ref();

    if to.is_dir() {
        return Err(CargoPlayError::PathExistError(to.to_path_buf()));
    }

    Command::new("cp")
        .arg("-R")
        .arg(from.as_ref())
        .arg(&to)
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .status()
        .map(|x| {
            // At this point we are certain the `to` path exists
            println!(
                "Generated project at {}",
                to.canonicalize().unwrap().display()
            );
            x
        })
        .map_err(From::from)
}
