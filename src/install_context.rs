use crate::{env::Env, Args};

#[derive(Debug)]
pub enum InstallContext {
    Project,
    Package,
}

impl InstallContext {
    pub fn suffix(&self, args: &Args) -> String {
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
