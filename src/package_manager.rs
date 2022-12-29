use crate::{env::node_env, Script};
use log::trace;
use neon::prelude::*;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug)]
pub enum PackageManager {
    NPM,
    Yarn,
    PNPM,
}

impl PackageManager {
    fn from_path(path: &Path) -> Option<Self> {
        if path.ends_with("npm") {
            Some(PackageManager::NPM)
        } else if path.ends_with("pnpm") {
            Some(PackageManager::PNPM)
        } else if path.ends_with("yarn") {
            Some(PackageManager::Yarn)
        } else {
            None
        }
    }

    pub fn from_node_env<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<Self> {
        let package_manager = node_env(cx, "_")?;
        trace!("_: {:?}", package_manager);

        let package_manager = Self::from_path(&PathBuf::from(&package_manager))
            .ok_or_else(|| {
                format!("Unable to resolve package manager from path: {package_manager}")
            })
            .or_else(|message| cx.throw_error(message))?;

        trace!("package_manager: {:?}", package_manager);

        Ok(package_manager)
    }

    pub(crate) fn run_script<'a, C: Context<'a>>(
        self,
        cx: &mut C,
        script: Script,
    ) -> NeonResult<()> {
        Command::new(self.to_string())
            .arg("run")
            .arg(script.to_string())
            .status()
            .map_err(|error| error.to_string())
            .and_then(|status| status.exit_ok().map_err(|error| error.to_string()))
            .or_else(|message| cx.throw_error(message))?;

        Ok(())
    }
}

impl ToString for PackageManager {
    fn to_string(&self) -> String {
        match self {
            Self::NPM => "npm",
            Self::PNPM => "pnpm",
            Self::Yarn => "yarn",
        }
        .to_string()
    }
}
