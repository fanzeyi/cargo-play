use base64;
use sha1;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::vec::Vec;
use structopt::StructOpt;

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
