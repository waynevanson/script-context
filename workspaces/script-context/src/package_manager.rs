use crate::{env::IntoEnvFromNode, from_error_result, script::Script};
use log::trace;
use neon::prelude::*;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
}

impl PackageManager {
    fn from_path(path: &Path) -> Option<Self> {
        if path.ends_with("npm") {
            Some(PackageManager::Npm)
        } else if path.ends_with("pnpm") {
            Some(PackageManager::Pnpm)
        } else if path.ends_with("yarn") {
            Some(PackageManager::Yarn)
        } else {
            None
        }
    }

    pub fn try_from_node_env<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<Self> {
        let package_manager = cx.into_env("_")?;

        let package_manager = Self::from_path(&PathBuf::from(&package_manager))
            .ok_or_else(|| {
                format!("Unable to resolve package manager from path: {package_manager}")
            })
            .or_else(from_error_result(cx))?;

        trace!("package_manager: {:?}", package_manager);

        Ok(package_manager)
    }

    pub fn run_script<'a, C: Context<'a>>(self, cx: &mut C, script: Script) -> NeonResult<()> {
        Command::new(self.to_string())
            .arg("run")
            .arg(script.to_string())
            .status()
            .or_else(from_error_result(cx))
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    from_error_result(cx)(status.to_string())
                }
            })?;

        Ok(())
    }
}

impl ToString for PackageManager {
    fn to_string(&self) -> String {
        match self {
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
        }
        .to_string()
    }
}
