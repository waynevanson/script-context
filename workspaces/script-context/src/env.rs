use crate::package_manager::PackageManager;
use log::trace;
use neon::prelude::*;
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
        let project_dir = project_dir(cx)?;
        let package_dir = package_dir(cx)?;
        let lifecycle_event = lifecycle_event(cx)?;
        let package_manager = PackageManager::from_node_env(cx)?;

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

fn package_dir<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<PathBuf> {
    let package_dir = node_env(cx, "PWD")
        .map(PathBuf::from)
        .and_then(|path| find_lowest_package_json_dir(cx, &path))?;

    trace!("package_dir: {:?}", package_dir);

    Ok(package_dir)
}

fn project_dir<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<PathBuf> {
    let init_cwd = node_env(cx, "INIT_CWD")?;
    trace!("init_cwd: {:?}", init_cwd);

    let project_dir = find_lowest_package_json_dir(cx, &PathBuf::from(init_cwd))?;
    trace!("project_dir: {:?}", project_dir);

    Ok(project_dir)
}

fn lifecycle_event<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<String> {
    let lifecycle_event = node_env(cx, "npm_lifecycle_event")?;

    trace!("lifecycle_event: {:?}", lifecycle_event);

    Ok(lifecycle_event)
}

pub fn node_env<'a, C: Context<'a>>(cx: &mut C, key: &str) -> NeonResult<String> {
    let global = cx.global();
    let process = global.get::<JsObject, _, _>(cx, "process")?;
    let version = process.get::<JsObject, _, _>(cx, "env")?;
    let var = version.get::<JsString, _, _>(cx, key)?.value(cx);

    Ok(var)
}

// find package json in string component
fn find_lowest_package_json_dir<'a, C: Context<'a>>(
    cx: &mut C,
    path: &Path,
) -> NeonResult<PathBuf> {
    let mut parent = Some(path);

    while let Some(directory) = parent {
        let package_json = directory.join("package.json");
        // try_exists?
        if let true = package_json.exists() {
            break;
        }

        parent = directory.parent();
    }

    let message = || {
        format!(
            "Unable to locate `package.json` file within any component of `{:?}`",
            path
        )
    };

    let parent = parent
        .ok_or_else(message)
        .or_else(|message| cx.throw_error(message))?;

    Ok(parent.to_owned())
}
