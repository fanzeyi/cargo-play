#![feature(type_alias_enum_variants)]

mod cargo;
mod errors;
mod opt;

use log::debug;
use opt::Opt;
use std::env::temp_dir;
use std::fs::File;
use std::io::{Read, Write};
use std::iter::Iterator;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::vec::Vec;
use structopt::StructOpt;

use crate::cargo::CargoManifest;
use crate::errors::CargoPlayError;

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
                .take_while(|line| line.starts_with("//#"))
                .map(|line| line[3..].trim_start().into())
                .filter(|s: &String| !s.is_empty())
                .collect()
        })
        .flatten()
        .collect()
}

fn mktemp(name: PathBuf) -> PathBuf {
    let mut temp = PathBuf::new();
    temp.push(temp_dir());
    temp.push(name);

    debug!("Creating temporary building folder at: {:?}", temp);
    if let Err(_) = std::fs::create_dir(&temp) {
        debug!("Temporary directory already exists.");
    }

    temp
}

fn write_cargo_toml(
    dir: &PathBuf,
    name: String,
    dependencies: Vec<String>,
) -> Result<(), CargoPlayError> {
    let manifest = CargoManifest::new(name, dependencies)?;
    let mut cargo = File::create(dir.join("Cargo.toml"))?;

    cargo.write_all(&toml::to_vec(&manifest).map_err(CargoPlayError::from_serde)?)?;

    Ok(())
}

fn copy_sources(dir: &PathBuf, sources: &Vec<PathBuf>) -> Result<(), CargoPlayError> {
    let src = dir.join("src");
    let _ = std::fs::create_dir(&src);
    let mut first = true;

    sources.iter().for_each(|file| {
        let filename = file.file_name().unwrap();
        let to = if first {
            first = true;
            src.join("main.rs")
        } else {
            src.join(filename)
        };

        debug!("Copying {:?} => {:?}", file, to);
        std::fs::copy(file, to).unwrap();
    });

    Ok(())
}

fn run_cargo_build(project: &PathBuf) -> Result<ExitStatus, CargoPlayError> {
    Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg(project.join("Cargo.toml"))
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .status()
        .map_err(From::from)
}

/// Remove the first element if it is "play" for cargo compatibility
fn trim_first_play<'a, T: Iterator<Item = U> + 'a, U: Into<String> + Clone + 'a>(
    mut input: T,
) -> Box<dyn Iterator<Item = T::Item> + 'a> {
    if let Some(first) = input.nth(0) {
        if first.clone().into() == "play" {
            Box::new(input)
        } else {
            Box::new(std::iter::once(first).chain(input))
        }
    } else {
        Box::new(input)
    }
}

fn main() -> Result<(), CargoPlayError> {
    let opt = Opt::from_iter(trim_first_play(std::env::args()));
    let files = parse_inputs(&opt.src)?;
    let dependencies = extract_headers(&files);
    let temp = mktemp(opt.temp_dirname());

    write_cargo_toml(&temp, opt.src_hash(), dependencies)?;
    copy_sources(&temp, &opt.src)?;

    match run_cargo_build(&temp)?.code() {
        Some(code) => std::process::exit(code),
        None => std::process::exit(-1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_first_play() {
        let testcases = vec![
            (vec!["play", "test1"], vec!["test1"]),
            (vec!["play", "play", "test2"], vec!["play", "test2"]),
            (vec!["test3"], vec!["test3"]),
            (vec![], vec![]),
        ];

        for (input, expected) in testcases {
            assert_eq!(
                trim_first_play(input.into_iter()).collect::<Vec<&str>>(),
                expected
            );
        }
    }

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
