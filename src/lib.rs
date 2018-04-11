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

pub struct NpmScripts {
    path: PathBuf,
}

impl NpmScripts {
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        NpmScripts { path: path.into() }
    }

    pub fn is_available(&self) -> bool {
        self.path.exists()
    }

    fn ensure_available(&self) -> Result<(), Error> {
        if !self.is_available() {
            bail!("no package.json found");
        }
        Ok(())
    }

    pub fn install(&self) -> Result<(), Error> {
        self.ensure_available()?;

        Command::new("npm")
            .arg("install")
            .current_dir(&self.path)
            .output()?;

        Ok(())
    }

    fn get_package_cfg(&self) -> Result<NpmPackageConfig, Error> {
        let mut file = File::open(&self.path.join("package.json"))?;
        let parsed = serde_json::from_reader(&mut file)?;
        Ok(parsed)
    }

    pub fn has_script(&self, script: &str) -> Result<bool, Error> {
        self.ensure_available()?;

        let cfg = self.get_package_cfg()?;

        Ok(cfg.scripts.is_some() && cfg.scripts.unwrap().contains_key(script))
    }

    pub fn run_script(&self, script: &str) -> Result<(), Error> {
        self.ensure_available()?;
        if let Err(err) = self.has_script(script) {
            return Err(err);
        }

        Command::new("npm")
            .arg("run")
            .arg(script)
            .current_dir(&self.path)
            .output()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_detects_missing_package_json() {
        let scripts = NpmScripts::new("./examples");
        let has_some = scripts.has_script(&"build".to_owned());

        assert!(has_some.is_err());
    }

    #[test]
    fn test_package_does_not_have_script() {
        let scripts = NpmScripts::new("./examples/ex1");
        let has_some = scripts.has_script("build").unwrap();

        assert!(!has_some);
    }

    #[test]
    fn test_package_has_script() {
        let scripts = NpmScripts::new("./examples/ex2");
        let has_some = scripts.has_script("build").unwrap();

        assert!(has_some);
    }

    #[test]
    fn test_running_a_script() {
        let scripts = NpmScripts::new("./examples/ex3");
        let res = scripts.run_script("build");

        assert!(res.is_ok());
    }

    #[test]
    fn it_installs_dependencies() {
        let scripts = NpmScripts::new("./examples/ex4");

        let res = scripts.install();

        assert!(res.is_ok());
    }
}
