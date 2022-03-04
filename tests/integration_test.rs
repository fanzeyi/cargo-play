use cargo_play::options::Options;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;
use std::ffi::OsStr;
use std::io::prelude::*;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};

struct TestRuntime {
    scratch: PathBuf,
}

impl TestRuntime {
    fn new() -> Result<Self> {
        let scratch = Self::create_scratch_dir()?;

        Ok(TestRuntime { scratch })
    }

    fn create_scratch_dir() -> Result<PathBuf> {
        let tmp = env::temp_dir();
        let scratch = tmp.join(Self::random_string());

        if scratch.exists() {
            let _ = std::fs::remove_dir_all(&scratch);
        }

        std::fs::create_dir(&scratch)?;

        Ok(scratch)
    }

    fn random_string() -> String {
        format!(
            "cargo-play-test.{}",
            thread_rng()
                .sample_iter(&Alphanumeric)
                .map(char::from)
                .take(10)
                .collect::<String>()
        )
    }

    fn temp_dir<I: AsRef<Path>>(&self, path: I) -> PathBuf {
        self.scratch.join(path)
    }

    fn run_with_stdin<
        I: IntoIterator<Item = S> + std::fmt::Debug,
        S: AsRef<OsStr> + std::fmt::Debug,
    >(
        &self,
        args: I,
    ) -> Command {
        let mut play = std::process::Command::new(cargo_play_binary_path());
        play.env("TMP", &self.scratch)
            .env("TMPDIR", &self.scratch)
            .args(args)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped());
        play
    }

    fn run<I: IntoIterator<Item = S> + std::fmt::Debug, S: AsRef<OsStr> + std::fmt::Debug>(
        &self,
        args: I,
    ) -> std::io::Result<StringOutput> {
        let mut play = std::process::Command::new(cargo_play_binary_path());
        play.env("TMP", &self.scratch)
            .env("TMPDIR", &self.scratch)
            .args(args)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
            .map(From::from)
    }
}

impl Drop for TestRuntime {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.scratch);
    }
}

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

#[test]
fn basic() -> Result<()> {
    let rt = TestRuntime::new()?;
    let output = rt.run(&["fixtures/hello.rs"])?;

    assert_eq!(output.status.code().unwrap(), 0);
    assert_eq!(output.stdout, "Hello World!\n");

    Ok(())
}

#[test]
fn clean() -> Result<()> {
    let rt = TestRuntime::new()?;
    let opt = Options::with_files(vec!["fixtures/hello.rs"]);
    let path = rt.temp_dir(opt.temp_dirname());
    let canary = path.clone().join("canary");

    if path.exists() {
        std::fs::remove_dir_all(&path)?;
    }

    println!("{:?}", path);
    let _ = rt.run(&["fixtures/hello.rs"])?;
    assert!(path.exists());

    std::fs::write(&canary, "I_AM_CANARY")?;

    assert!(canary.exists());
    let _ = rt.run(&["--clean", "fixtures/hello.rs"])?;
    assert!(!canary.exists());

    Ok(())
}

#[test]
fn edition() -> Result<()> {
    let rt = TestRuntime::new()?;

    // default edition is 2021
    let output = rt.run(&["fixtures/edition.rs"])?;
    assert_ne!(output.status.code().unwrap(), 0);

    let output = rt.run(&["--edition", "2021", "fixtures/edition.rs"])?;
    assert_ne!(output.status.code().unwrap(), 0);

    let output = rt.run(&["--edition", "2018", "fixtures/edition.rs"])?;
    assert_ne!(output.status.code().unwrap(), 0);

    // it should pass in 2015
    let output = rt.run(&["--edition", "2015", "fixtures/edition.rs"])?;
    assert_eq!(output.status.code().unwrap(), 0);

    Ok(())
}

#[test]
fn debug_mode() -> Result<()> {
    let rt = TestRuntime::new()?;

    let opt = Options::with_files(vec!["fixtures/hello.rs"]);
    let path = rt.temp_dir(opt.temp_dirname());

    let _ = rt.run(&["fixtures/hello.rs"])?;
    assert!(path.join("target").join("debug").exists());
    assert!(!path.join("target").join("release").exists());

    Ok(())
}

#[test]
fn release_mode() -> Result<()> {
    let rt = TestRuntime::new()?;

    let opt = Options::with_files(vec!["fixtures/hello.rs"]);
    let path = rt.temp_dir(opt.temp_dirname());

    let _ = rt.run(&["--release", "fixtures/hello.rs"])?;
    assert!(!path.join("target").join("debug").exists());
    assert!(path.join("target").join("release").exists());

    Ok(())
}

#[test]
fn quiet_mode() -> Result<()> {
    let rt = TestRuntime::new()?;
    let output = rt.run(&["--quiet", "fixtures/hello.rs"])?;
    assert!(!output.stderr.contains("Running"));
    Ok(())
}

#[test]
fn verbose_mode() -> Result<()> {
    let rt = TestRuntime::new()?;
    let output = rt.run(&["-v", "fixtures/hello.rs"])?;
    assert!(output.stderr.contains("rustc"));
    Ok(())
}

#[test]
fn cargo_option() -> Result<()> {
    let rt = TestRuntime::new()?;

    let opt = Options::with_files(vec!["fixtures/hello.rs"]);
    let path = rt.temp_dir(opt.temp_dirname());

    let _ = rt.run(&["--cargo-option=--release", "fixtures/hello.rs"])?;

    assert!(!path.join("target").join("debug").exists());
    assert!(path.join("target").join("release").exists());

    Ok(())
}

#[test]
fn program_args() -> Result<()> {
    let rt = TestRuntime::new()?;

    let output = rt.run(&["fixtures/args.rs", "--", "test"])?;
    assert_eq!(output.stdout, "test\n");

    Ok(())
}

#[test]
fn external_crate() -> Result<()> {
    let rt = TestRuntime::new()?;

    let output = rt.run(&["fixtures/bitflags.rs"])?;
    assert_eq!(output.status.code().unwrap(), 0);

    Ok(())
}

#[test]
fn simple_infer() -> Result<()> {
    let rt = TestRuntime::new()?;
    let output = rt.run(&["--infer", "fixtures/infer.rs"])?;
    assert_eq!(output.status.code().unwrap(), 0);

    Ok(())
}

#[test]
fn infer_failure() -> Result<()> {
    let rt = TestRuntime::new()?;
    let output = rt.run(&["--infer", "fixtures/infer-failure.rs"])?;
    assert_ne!(output.status.code().unwrap(), 0);

    Ok(())
}

#[test]
fn infer_override() -> Result<()> {
    let rt = TestRuntime::new()?;
    let output = rt.run(&["--infer", "fixtures/infer-override.rs"])?;
    assert_eq!(output.status.code().unwrap(), 0);

    Ok(())
}

/// See https://github.com/fanzeyi/cargo-play/pull/13 for details
#[test]
fn dtoa_test() -> Result<()> {
    let rt = TestRuntime::new()?;
    let output = rt.run(&["fixtures/dtoa.rs"])?;
    assert_eq!(dbg!(output).status.code().unwrap(), 0);

    Ok(())
}

#[test]
fn test_mode_test() -> Result<()> {
    let rt = TestRuntime::new()?;
    let output = rt.run(&["--test", "fixtures/tests.rs"])?;
    println!("{}", output.stderr);
    assert_eq!(output.status.code().unwrap(), 0);

    Ok(())
}

#[test]
fn stdin_with_hello() -> Result<()> {
    let rt = TestRuntime::new()?;
    let mut ps = {
        let mut p = rt.run_with_stdin(&["--stdin", "mod_hello.rs"]);
        p.current_dir(std::fs::canonicalize("fixtures")?);
        p.spawn()?
    };
    {
        let mut stdin = ps.stdin.take().unwrap();
        stdin.write_all("mod mod_hello; fn main() { mod_hello::hello(); }".as_bytes())?;
    } // close stdin
    let status = ps.wait()?;
    assert_eq!(status.code().unwrap(), 0);
    let out = {
        let mut buff = String::new();
        ps.stdout.unwrap().read_to_string(&mut buff)?;
        buff
    };
    assert_eq!(out, "Hello World!\n");

    Ok(())
}
