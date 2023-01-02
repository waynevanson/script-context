mod env;
mod install_context;
mod package_json;
mod package_manager;
use crate::{env::Env, package_json::PackageJson};
use clap::Parser;
use log::{error, trace, warn, Level};
use neon::prelude::*;
use std::env::args_os;

pub use crate::install_context::InstallContext;

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

#[derive(Parser, Debug, PartialEq, Eq)]
struct Args {
    #[arg(short, long, default_value = ":")]
    delimiter: char,
    #[arg(long, default_value = "project")]
    project: String,
    #[arg(long, default_value = "package")]
    package: String,
}

fn from_error_result<'a, C, E, T>(cx: &mut C) -> impl FnOnce(E) -> NeonResult<T> + '_
where
    C: Context<'a>,
    E: ToString,
{
    move |error: E| cx.throw_error(error.to_string())
}

impl Args {
    fn from_node<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<Self> {
        Self::try_parse_from(args_os().skip(1)).or_else(from_error_result(cx))
    }
}

fn cli(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let args = Args::from_node(&mut cx)?;
    let env = Env::from_node_env(&mut cx)?;

    let install_context = InstallContext::from(&env);

    let package_json =
        PackageJson::from_dir(&env.package_dir).or_else(from_error_result(&mut cx))?;

    trace!("package_json: {:?}", package_json);

    let script = Script {
        lifecycle: env.lifecycle_event,
        delimiter: args.delimiter,
        suffix: install_context.suffix(&args).clone(),
    };

    trace!("script: {:?}", script);

    let script_exists = package_json.script_exists(&script);

    if !script_exists {
        let message = format!("{} install script not found", script.to_string());

        match install_context {
            InstallContext::Project => warn!("{message}, skipping within the project"),
            InstallContext::Package => {
                let message = format!("{message}, required as a package dependency");
                error!("{message}");
                cx.throw_error(message)?;
            }
        };
    } else {
        env.package_manager.run_script(&mut cx, script)?;
    };

    Ok(cx.undefined())
}

fn context(mut cx: FunctionContext) -> JsResult<JsString> {
    let context = InstallContext::from_node_env(&mut cx)?;

    Ok(cx.string(context))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    simple_logger::init_with_level(Level::Trace).or_else(from_error_result(&mut cx))?;

    cx.export_function("cli", cli)?;

    cx.export_function("installContext", context)?;

    Ok(())
}
