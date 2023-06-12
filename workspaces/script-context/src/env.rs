use crate::{from_error_result, package_manager::PackageManager};
use log::trace;
use neon::{object::PropertyKey, prelude::*};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Env {
    pub project_dir: PathBuf,
    pub package_dir: PathBuf,
    pub lifecycle_event: String,
    pub package_manager: PackageManager,
}

impl Env {
    pub fn from_node_env<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<Self> {
        let project_dir = cx
            .into_env("INIT_CWD")
            .map(PathBuf::from)
            .and_then(|pathbuf| {
                pathbuf
                    .find_lowest_file("package.json")
                    .or_else(from_error_result(cx))
            })?;

        let package_dir = cx.into_env("PWD").map(PathBuf::from).and_then(|pathbuf| {
            pathbuf
                .find_lowest_file("package.json")
                .or_else(from_error_result(cx))
        })?;

        let lifecycle_event = cx.into_env("npm_lifecycle_event")?;
        let package_manager = PackageManager::try_from_node_env(cx)?;

        let env = Self {
            lifecycle_event,
            package_dir,
            project_dir,
            package_manager,
        };

        trace!("env: {:?}", env);

        Ok(env)
    }
}

pub trait IntoEnvFromNode {
    fn into_env(&mut self, key: impl PropertyKey + Copy + ToString) -> NeonResult<String>;
}

impl<'a, C> IntoEnvFromNode for C
where
    C: Context<'a>,
{
    fn into_env(&mut self, key: impl PropertyKey + Copy + ToString) -> NeonResult<String> {
        let var = self
            .global()
            .get::<JsObject, _, _>(self, "process")?
            .get::<JsObject, _, _>(self, "env")?
            .get::<JsString, _, _>(self, key)?
            .value(self);

        trace!("{}: {:?}", key.to_string(), var);
        Ok(var)
    }
}

trait FindLowestFile {
    fn find_lowest_file(&self, file_name: impl AsRef<str>) -> Result<PathBuf, String>;
}

impl FindLowestFile for Path {
    fn find_lowest_file(&self, file_name: impl AsRef<str>) -> Result<PathBuf, String> {
        let mut parent = Some(self);

        while let Some(directory) = parent {
            let package_json = directory.join(file_name.as_ref());

            // try_exists?
            if let true = package_json.exists() {
                break;
            }

            parent = directory.parent();
        }

        parent
            .ok_or_else(|| {
                format!(
                    "Unable to locate `{}` file within any component of `{:?}`",
                    file_name.as_ref(),
                    self
                )
            })
            .map(|path| path.to_owned())
    }
}
