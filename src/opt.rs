use std::ffi::{OsStr, OsString};
use std::iter::FromIterator;
use std::path::PathBuf;
use std::str::FromStr;
use std::vec::Vec;
use structopt::StructOpt;

use crate::errors::CargoPlayError;

#[derive(Debug)]
pub(crate) enum RustEdition {
    E2015,
    E2018,
}

impl FromStr for RustEdition {
    type Err = CargoPlayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "2018" {
            Ok(RustEdition::E2018)
        } else if s == "2015" {
            Ok(RustEdition::E2015)
        } else {
            Err(CargoPlayError::InvalidEdition(s.into()))
        }
    }
}

impl Into<String> for RustEdition {
    fn into(self) -> String {
        match self {
            RustEdition::E2015 => "2015".into(),
            RustEdition::E2018 => "2018".into(),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "cargo-play", about = "Single file cargo runner.")]
pub(crate) struct Opt {
    #[structopt(short = "d", long = "debug", hidden = true)]
    debug: bool,
    #[structopt(short = "c", long = "clean")]
    pub clean: bool,
    #[structopt(short = "t", long = "toolchain", hidden = true)]
    pub toolchain: Option<String>,
    #[structopt(
        parse(try_from_os_str = "osstr_to_abspath"),
        raw(required = "true", validator = "file_exist")
    )]
    pub src: Vec<PathBuf>,
    #[structopt(
        short = "e",
        long = "edition",
        default_value = "2018",
        raw(possible_values = r#"&["2015", "2018"]"#)
    )]
    pub edition: RustEdition,
    #[structopt(long = "release")]
    pub release: bool,
    #[structopt(long = "cached")]
    pub cached: bool,
    #[structopt(short = "a", long = "arg", multiple = true)]
    pub args: Vec<String>,
}

impl Opt {
    /// Generate a string of hash based on the path passed in
    pub fn src_hash(&self) -> String {
        let mut hash = sha1::Sha1::new();
        let mut srcs = self.src.clone();

        srcs.sort();

        for file in srcs.into_iter() {
            hash.update(file.to_string_lossy().as_bytes());
        }

        base64::encode_config(&hash.digest().bytes()[..], base64::URL_SAFE_NO_PAD)
    }

    pub fn temp_dirname(&self) -> PathBuf {
        format!("cargo-play.{}", self.src_hash()).into()
    }

    fn with_toolchain(mut self, toolchain: Option<String>) -> Self {
        self.toolchain = toolchain;
        self
    }

    pub fn parse(args: Vec<String>) -> Result<Self, ()> {
        if args.len() < 2 {
            Self::clap().print_help().unwrap_or(());
            return Err(());
        }

        let with_cargo = args[1] == "play";
        let mut args = args.into_iter();

        if with_cargo {
            args.next();
        }

        let toolchain = args
            .clone()
            .find(|x| x.starts_with("+"))
            .map(|s| String::from_iter(s.chars().skip(1)));

        Ok(Opt::from_iter(args.filter(|x| !x.starts_with("+"))).with_toolchain(toolchain))
    }
}

/// Convert `std::ffi::OsStr` to an absolute `std::path::PathBuf`
fn osstr_to_abspath(v: &OsStr) -> Result<PathBuf, OsString> {
    if let Ok(r) = PathBuf::from(v).canonicalize() {
        Ok(r)
    } else {
        Err(v.into())
    }
}

/// structopt compataible function to check whether a file exists
fn file_exist(v: String) -> Result<(), String> {
    let p = PathBuf::from(v);
    if !p.is_file() {
        Err(format!("input file does not exist: {:?}", p))
    } else {
        Ok(())
    }
}
