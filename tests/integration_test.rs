use cargo_play::opt::Opt;
use cargo_play::steps;
use std::env;
use std::ffi::OsStr;
use std::io::Result;
use std::path::PathBuf;
use std::process::{ExitStatus, Output, Stdio};

fn cargo_play_binary_path() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    let exe = String::from("cargo-play") + env::consts::EXE_SUFFIX;
    path.push(exe);
    path
}

#[derive(Debug)]
struct StringOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

impl From<std::process::Output> for StringOutput {
    fn from(v: Output) -> Self {
        StringOutput {
            status: v.status,
            stdout: String::from_utf8_lossy(&v.stdout).to_string(),
            stderr: String::from_utf8_lossy(&v.stderr).to_string(),
        }
    }
}

fn cargo_play<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
    args: I,
) -> std::io::Result<StringOutput> {
    let mut play = std::process::Command::new(cargo_play_binary_path());
    play.args(args)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .map(From::from)
}

#[test]
fn basic_compile() -> Result<()> {
    let output = cargo_play(&["-c", "fixtures/hello.rs"])?;

    assert_eq!(output.status.code().unwrap(), 0);
    assert_eq!(output.stdout, "Hello World!\n");

    Ok(())
}

#[test]
fn clean() -> Result<()> {
    let opt = Opt {
        src: vec![PathBuf::from("fixtures/hello.rs").canonicalize()?],
        ..Default::default()
    };
    let path = steps::temp_dir(opt.temp_dirname());
    let canary = path.clone().join("canary");

    if path.exists() {
        std::fs::remove_dir_all(&path)?;
    }

    println!("{:?}", path);
    let _ = dbg!(cargo_play(&["fixtures/hello.rs"])?);
    assert!(path.exists());

    std::fs::write(&canary, "I_AM_CANARY")?;

    assert!(canary.exists());
    let _ = cargo_play(&["--clean", "fixtures/hello.rs"])?;
    assert!(!canary.exists());

    Ok(())
}
