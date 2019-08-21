mod cargo;
mod errors;
mod infer;
mod opt;
mod steps;

use std::collections::HashSet;
use std::iter::Iterator;
use std::process::{Command, Stdio};
use std::vec::Vec;

use crate::errors::CargoPlayError;
use crate::opt::Opt;
use crate::steps::*;

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

    let infers = if opt.infer {
        infer::analyze_sources(&opt.src)?
    } else {
        HashSet::new()
    };

    if opt.clean {
        rmtemp(&temp);
    }
    mktemp(&temp);
    write_cargo_toml(&temp, src_hash.clone(), dependencies, opt.edition, infers)?;
    copy_sources(&temp, &opt.src)?;

    let end = if let Some(save) = opt.save {
        copy_project(&temp, &save)?
    } else {
        run_cargo_build(
            opt.toolchain,
            &temp,
            opt.release,
            opt.cargo_option,
            &opt.args,
        )?
    };

    match end.code() {
        Some(code) => std::process::exit(code),
        None => std::process::exit(-1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_headers() {
        let inputs: Vec<String> = vec![
            r#"//# line 1
//# line 2
// line 3
//# line 4"#,
        ]
        .into_iter()
        .map(Into::into)
        .collect();
        let result = extract_headers(&inputs);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], String::from("line 1"));
        assert_eq!(result[1], String::from("line 2"));
    }
}
