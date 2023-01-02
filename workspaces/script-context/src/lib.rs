mod env;
mod install_context;
mod package_json;
mod package_manager;
mod script;

use std::env::args_os;

use clap::Parser;
use neon::prelude::*;

pub use crate::{
    env::Env, install_context::InstallContext, package_json::PackageJson,
    package_manager::PackageManager, script::Script,
};

#[derive(Parser, Debug, PartialEq, Eq)]
pub struct Args {
    #[arg(short, long, default_value = ":")]
    pub delimiter: char,
    #[arg(long, default_value = "project")]
    pub project: String,
    #[arg(long, default_value = "package")]
    pub package: String,
}

impl Args {
    pub fn from_node<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<Self> {
        Self::try_parse_from(args_os().skip(1)).or_else(from_error_result(cx))
    }
}

fn from_error_result<'a, C, E, T>(cx: &mut C) -> impl FnOnce(E) -> NeonResult<T> + '_
where
    C: Context<'a>,
    E: ToString,
{
    move |error: E| cx.throw_error(error.to_string())
}
