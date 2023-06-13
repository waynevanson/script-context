use log::{error, info, trace, Level};
use neon::prelude::*;
use script_context::{Env, InstallContext, PackageJson, Script, TryIntoArgs};

fn from_error_result<'a, C, E, T>(cx: &mut C) -> impl FnOnce(E) -> NeonResult<T> + '_
where
    C: Context<'a>,
    E: ToString,
{
    move |error: E| cx.throw_error(error.to_string())
}

fn cli(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let args = cx.try_into_args()?;
    let env = Env::from_node_env(&mut cx)?;

    let install_context = InstallContext::from(&env);

    let package_json =
        PackageJson::from_dir(&env.package_dir).or_else(from_error_result(&mut cx))?;

    let script = Script {
        lifecycle: env.lifecycle_event,
        delimiter: args.delimiter,
        suffix: install_context.suffix(&args).clone(),
    };

    trace!("{:?}", script);

    let script_exists = package_json.script_exists(&script);

    if !script_exists {
        let message = format!("{} script not found", script.to_string());

        match install_context {
            InstallContext::Project => info!("{message}, skipping within the project"),
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

fn install_context(mut cx: FunctionContext) -> JsResult<JsString> {
    let context = InstallContext::try_from_node_env(&mut cx)?;

    Ok(cx.string(context))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    simple_logger::init_with_level(Level::Trace).or_else(from_error_result(&mut cx))?;

    cx.export_function("cli", cli)?;

    cx.export_function("installContext", install_context)?;

    Ok(())
}
