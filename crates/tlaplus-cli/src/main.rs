#![forbid(unsafe_code)]
#![deny(clippy::all)]

mod config;
mod manifest;

mod translate;
mod update;

use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use clap::StructOpt;

#[derive(clap::Parser)]
#[non_exhaustive]
enum Opt {
    #[clap(alias = "u")]
    Update,
    #[clap(alias = "t")]
    Translate(translate::Opt),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opt = Opt::parse();

    match opt {
        Opt::Update => update::run().await?,
        Opt::Translate(opt) => translate::run(opt).await?,
    }

    Ok(())
}

pub(crate) fn exec_tla2tools(args: Vec<&str>) -> Result<()> {
    use crate::config::Config;
    use crate::manifest::Manifest;

    let manifest = Manifest::load()?;
    let config = Config::load()?;

    let jar_path = manifest
        .tla2tools_current_path()
        .with_context(|| "Could not find tla2tools. Please update.")?;

    let argv = {
        let mut v = vec!["java"];

        v.push("-cp");
        v.push(jar_path.as_ref());

        if let Some(ref java_config) = config.java {
            v.extend(java_config.args.iter().map(|s| s.as_str()));
        }

        v.extend(args);

        v
    };

    let mut cmd = Command::new(argv[0]);

    cmd.args(&argv[1..]);

    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    println!("{:?}", cmd);

    let exit_status = cmd.spawn()?.wait()?;
    if !exit_status.success() {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::prelude::ExitStatusExt;
            bail!(
                "pcal.trans failed. code = {:?}, signal = {:?}",
                exit_status.code(),
                exit_status.signal()
            );
        }
        #[cfg(not(target_os = "linux"))]
        {
            todo!()
        }
    }

    Ok(())
}
