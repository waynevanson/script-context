#![feature(exit_status_error)]

use clap::Parser;
use log::{error, trace, warn, Level};
use neon::prelude::*;
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::args_os,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug)]
struct Script {
    lifecycle: String,
    delimiter: char,
    suffix: String,
}

impl ToString for Script {
    fn to_string(&self) -> String {
        self.lifecycle.to_string() + &self.delimiter.to_string() + &self.suffix
    }
}

#[derive(Deserialize, Debug)]
struct PackageJson {
    scripts: HashMap<String, String>,
}

impl PackageJson {
    fn from_dir(directory: &Path) -> Result<Self, String> {
        let contents =
            fs::read(directory.join("package.json")).map_err(|error| error.to_string())?;

        let package_json =
            serde_json::from_slice::<PackageJson>(&contents).map_err(|error| error.to_string())?;

        Ok(package_json)
    }
}

impl PackageJson {
    fn script_exists(&self, script: &Script) -> bool {
        self.scripts.get(&script.to_string()).is_some()
    }
}

fn node_env<'a, C: Context<'a>>(cx: &mut C, key: &str) -> NeonResult<String> {
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

    let message = format!(
        "Unable to locate `package.json` file within any component of `{:?}`",
        path
    );

    let parent = parent
        .ok_or(message)
        .or_else(|message| cx.throw_error(message))?;

    Ok(parent.to_owned())
}

#[derive(Parser, Debug, PartialEq, Eq)]
struct Args {
    #[arg(short, long, default_value = ":")]
    delimiter: char,
    #[arg(long, default_value = "project")]
    project: String,
    #[arg(long, default_value = "package")]
    package: String,
}

fn args<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<Args> {
    Args::try_parse_from(args_os().skip(1))
        .map_err(|error| error.to_string())
        .or_else(|message| cx.throw_error(message))
}

#[derive(Debug)]
enum PackageManager {
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

fn package_manager<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<PackageManager> {
    let package_manager = node_env(cx, "_")?;
    trace!("_: {:?}", package_manager);

    let package_manager = PackageManager::from_path(&PathBuf::from(&package_manager))
        .ok_or_else(|| format!("Unable to resolve package manager from path: {package_manager}"))
        .or_else(|message| cx.throw_error(message))?;

    trace!("package_manager: {:?}", package_manager);

    Ok(package_manager)
}

#[derive(Debug)]
enum InstallContext {
    Project,
    Package,
}

impl InstallContext {
    fn suffix(&self, args: &Args) -> String {
        match self {
            Self::Project => &args.project,
            Self::Package => &args.package,
        }
        .to_string()
    }
}

impl From<&Env> for InstallContext {
    fn from(env: &Env) -> Self {
        if env.project_dir == env.package_dir {
            Self::Project
        } else {
            Self::Package
        }
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

#[derive(Debug)]
struct Env {
    pub project_dir: PathBuf,
    pub package_dir: PathBuf,
    pub lifecycle_event: String,
    pub package_manager: PackageManager,
}

fn env<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<Env> {
    let project_dir = project_dir(cx)?;
    let package_dir = package_dir(cx)?;
    let lifecycle_event = lifecycle_event(cx)?;
    let package_manager = package_manager(cx)?;

    let env = Env {
        lifecycle_event,
        package_dir,
        project_dir,
        package_manager,
    };

    trace!("env: {:?}", env);

    Ok(env)
}

fn cli(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let args = args(&mut cx)?;
    let env = env(&mut cx)?;

    let install_context = InstallContext::from(&env);

    let package_json =
        PackageJson::from_dir(&env.package_dir).or_else(|message| cx.throw_error(message))?;

    trace!("package_json: {:?}", package_json);

    let script = Script {
        lifecycle: env.lifecycle_event,
        delimiter: args.delimiter,
        suffix: install_context.suffix(&args).clone(),
    };

    trace!("script: {:?}", script);

    let script_name = script.to_string();
    let script_exists = package_json.script_exists(&script);

    if let false = script_exists {
        let message = format!("{script_name} install script not found");

        match install_context {
            InstallContext::Project => warn!("{message}"),
            // throw error if this happens
            InstallContext::Package => error!("{message}"),
        };
    }

    if let true = script_exists {
        // run script
        Command::new(env.package_manager.to_string())
            .arg("run")
            .arg(script_name)
            .status()
            .map_err(|error| error.to_string())
            .and_then(|status| status.exit_ok().map_err(|error| error.to_string()))
            .or_else(|message| cx.throw_error(message))?;
    }

    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    simple_logger::init_with_level(Level::Trace)
        .map_err(|error| error.to_string())
        .or_else(|message| cx.throw_error(message))?;

    cx.export_function("cli", cli)?;

    Ok(())
}
