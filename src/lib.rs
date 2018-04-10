#[macro_use]
extern crate failure;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

use failure::Error;

#[derive(Deserialize)]
struct NpmPackageConfig {
    scripts: Option<HashMap<String, String>>,
}

fn get_package_cfg(path: &PathBuf) -> Result<NpmPackageConfig, Error> {
    let mut file = File::open(path)?;
    let parsed = serde_json::from_reader(&mut file)?;
    Ok(parsed)
}

pub fn has_scripts(path: &PathBuf) -> Result<bool, Error> {
    let cfg = get_package_cfg(path)?;

    Ok(cfg.scripts.is_some() && !cfg.scripts.unwrap().is_empty())
}

pub fn has_script(path: &PathBuf, script: &String) -> Result<bool, Error> {
    let path = path.join("package.json");
    if !path.exists() {
        format_err!("no package.json found at {:?}", path);
    }

    let cfg = get_package_cfg(&path)?;

    Ok(cfg.scripts.is_some() && cfg.scripts.unwrap().contains_key(script))
}

pub fn run_script(path: &PathBuf, script: &String) -> Result<(), Error> {
    if let Err(err) = has_script(&path, script) {
        return Err(err);
    }
    Command::new("npm")
        .arg("run")
        .arg(script)
        .current_dir(&path)
        .output()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_detects_missing_package_json() {
        let has_some = has_scripts(&PathBuf::from("./examples"));

        assert!(has_some.is_err());
    }

    #[test]
    fn it_reads_package_file_without_scripts() {
        let has_some = has_scripts(&PathBuf::from("./examples/contacts.package.json")).unwrap();

        assert!(!has_some);
    }

    #[test]
    fn it_reads_package_file_with_scripts() {
        let has_some = has_scripts(&PathBuf::from("./examples/mail.package.json")).unwrap();

        assert!(has_some);
    }

    #[test]
    fn test_package_has_script() {
        let has_some = has_script(&PathBuf::from("./examples"), &"build".to_owned()).unwrap();

        assert!(has_some);
    }

    #[test]
    fn test_package_does_not_have_script() {
        let has_some = has_script(&PathBuf::from("./examples"), &"blockchain".to_owned()).unwrap();

        assert!(!has_some);
    }

    #[test]
    fn test_running_a_script() {
        let res = run_script(&PathBuf::from("./examples"), &"build".to_owned());

        println!("{:?}", res);

        assert!(res.is_ok());
    }
}
