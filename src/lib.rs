#![feature(exit_status_error)]
mod env;
mod install_context;
mod package_json;
mod package_manager;
use crate::{env::Env, install_context::InstallContext, package_json::PackageJson};
use clap::Parser;
use log::{error, trace, warn, Level};
use neon::prelude::*;
use std::{env::args_os, process::Command};

#[derive(Debug)]
pub struct Script {
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
pub struct Args {
    #[arg(short, long, default_value = ":")]
    delimiter: char,
    #[arg(long, default_value = "project")]
    project: String,
    #[arg(long, default_value = "package")]
    package: String,
}

impl Args {
    fn from_node<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<Args> {
        Args::try_parse_from(args_os().skip(1))
            .map_err(|error| error.to_string())
            .or_else(|message| cx.throw_error(message))
    }
}

fn cli(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let args = Args::from_node(&mut cx)?;
    let env = Env::from_node_env(&mut cx)?;

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

    cx.export_function("installContext", context)?;

    Ok(())
}

fn context(mut cx: FunctionContext) -> JsResult<JsString> {
    let context = InstallContext::from_node_env(&mut cx)?;

    Ok(cx.string(context))
}
