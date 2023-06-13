use crate::script::Script;
use log::trace;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Deserialize, Debug)]
pub struct PackageJson {
    scripts: HashMap<String, String>,
}

impl PackageJson {
    pub fn from_dir(directory: &Path) -> Result<Self, String> {
        let contents =
            fs::read(directory.join("package.json")).map_err(|error| error.to_string())?;

        let package_json =
            serde_json::from_slice::<PackageJson>(&contents).map_err(|error| error.to_string())?;

        trace!("{:?}", package_json);

        Ok(package_json)
    }

    pub fn script_exists(&self, script: &Script) -> bool {
        self.scripts.get(&script.to_string()).is_some()
    }
}
