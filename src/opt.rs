use base64;
use sha1;
use std::ffi::{OsStr, OsString};
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
            Ok(Self::E2018)
        } else if s == "2015" {
            Ok(Self::E2015)
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
    #[structopt(short = "t", long = "toolchain", hidden = true)]
    toolchain: Option<String>,
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
}

impl Opt {
    /// Generate a string of hash based on the path passed in
    pub fn src_hash(&self) -> String {
        let mut hash = sha1::Sha1::new();

        for file in self.src.iter() {
            hash.update(file.to_string_lossy().as_bytes());
        }

        base64::encode_config(&hash.digest().bytes()[..], base64::URL_SAFE_NO_PAD)
    }

    pub fn temp_dirname(&self) -> PathBuf {
        format!("cargo-play.{}", self.src_hash()).into()
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
