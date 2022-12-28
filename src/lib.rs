use clap::Parser;
use log::{debug, error, trace, warn, Level};
use neon::prelude::*;
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::args_os,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct Script {
    script: String,
    delimiter: char,
    name: String,
}

impl ToString for Script {
    fn to_string(&self) -> String {
        self.script.to_string() + &self.delimiter.to_string() + &self.name
    }
}

fn vec_to_array_string<'a, C: Context<'a>>(cx: &mut C, vec: Vec<String>) -> JsResult<'a, JsArray> {
    let a = JsArray::new(cx, vec.len() as u32);
    for (i, s) in vec.iter().enumerate() {
        let v = cx.string(s);
        a.set(cx, i as u32, v)?;
    }
    Ok(a)
}

#[derive(Deserialize)]
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
    #[arg(long, default_value = "local")]
    local: String,
    #[arg(long, default_value = "dependency")]
    dependency: String,
}

fn args<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<Args> {
    Args::try_parse_from(args_os().skip(1))
        .map_err(|error| error.to_string())
        .or_else(|message| cx.throw_error(message))
}

enum PackageManager {
    NPM,
    Yarn,
    PNPM,
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

    if package_manager.ends_with("npm") {
        Ok(PackageManager::NPM)
    } else if package_manager.ends_with("pnpm") {
        Ok(PackageManager::PNPM)
    } else if package_manager.ends_with("yarn") {
        Ok(PackageManager::Yarn)
    } else {
        cx.throw_error("Could not find a package manager")
    }
}

#[derive(Debug)]
enum InstallContext {
    Local,
    Dependency,
}

impl InstallContext {
    fn from_paths(project: &Path, package: &Path) -> Self {
        if project == package {
            Self::Local
        } else {
            Self::Dependency
        }
    }
}

#[derive(Debug)]
struct CliParameters {
    pub lifecycle: String,
    pub project_dir: PathBuf,
    pub package_dir: PathBuf,
}

fn do_spawn<'a, C: Context<'a>>(
    cx: &mut C,
    f: Handle<'a, JsFunction>,
    command: String,
    vec: Vec<String>,
) -> JsResult<'a, JsValue> {
    let command = cx.string(command);
    let args = vec_to_array_string(cx, vec)?;
    let spawned: Handle<JsValue> = f.call_with(cx).arg(command).arg(args).apply(cx)?;

    Ok(spawned)
}

fn cli_parameters<'a>(
    cx: &mut FunctionContext<'a>,
) -> NeonResult<(CliParameters, Handle<'a, JsFunction>)> {
    let param = cx.argument::<JsObject>(0)?;
    let dirs: Handle<JsObject> = param.get(cx, "dir")?;

    let project_dir: PathBuf = dirs.get::<JsString, _, _>(cx, "project")?.value(cx).into();
    let package_dir: PathBuf = dirs.get::<JsString, _, _>(cx, "package")?.value(cx).into();

    let lifecycle: String = param.get::<JsString, _, _>(cx, "lifecycle")?.value(cx);

    let spawn: Handle<JsFunction> = param.get(cx, "spawn")?;

    let params = CliParameters {
        lifecycle,
        project_dir,
        package_dir,
    };

    trace!("{:?}", params);

    Ok((params, spawn))
}

fn cli(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let (params, spawn) = cli_parameters(&mut cx)?;

    let project_dir = find_lowest_package_json_dir(&mut cx, &params.project_dir)?;
    let package_dir = find_lowest_package_json_dir(&mut cx, &params.package_dir)?;
    let lifecycle = params.lifecycle;

    let args = args(&mut cx)?;
    let delimiter = args.delimiter;

    debug!("project_dir: {:?}", project_dir);
    debug!("package_dir: {:?}", package_dir);

    let install_context = InstallContext::from_paths(&project_dir, &package_dir);

    let name = match install_context {
        InstallContext::Local => args.local,
        InstallContext::Dependency => args.dependency,
    };

    let package_json =
        PackageJson::from_dir(&package_dir).or_else(|message| cx.throw_error(message))?;

    let script = Script {
        script: lifecycle,
        delimiter,
        name,
    };

    let script_name = script.to_string();
    let script_exists = package_json.script_exists(&script);

    if let false = script_exists {
        match install_context {
            InstallContext::Local => warn!("Local install script not found"),
            // throw error if this happens
            InstallContext::Dependency => error!("Dependency install script not found"),
        };
    }

    let package_manager = package_manager(&mut cx)?.to_string();

    if let true = script_exists {
        do_spawn(
            &mut cx,
            spawn,
            package_manager,
            vec!["run".to_string(), script_name],
        )?;
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
