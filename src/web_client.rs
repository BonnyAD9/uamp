use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use crate::{
    cli::run_detached,
    core::{
        AnyControlMsg, Error, Result, config::Config, server::client::Client,
    },
};

pub fn run_web_client(conf: &Config, init: Vec<AnyControlMsg>) -> Result<()> {
    let address = format!("{}:{}", conf.server_address(), conf.port());
    let is_running = {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(async {
            let Ok(mut client) = Client::connect(address.clone()).await else {
                return Result::Ok(false);
            };

            client.send_ctrl(&init).await?;

            Ok(true)
        })?
    };

    if !is_running {
        let (adr, port) = if conf.config_path.is_none() {
            (Some(conf.server_address().as_str()), Some(conf.port()))
        } else {
            (None, None)
        };
        run_detached(&init, adr, port)?;

        eprintln!("Waiting 1s for the server to startup.");
        thread::sleep(Duration::from_secs(1));
    }

    open_web(conf)
}

fn open_web(conf: &Config) -> Result<()> {
    let address =
        format!("http://{}:{}/app", conf.server_address(), conf.port());
    let Some(cmd) = conf.web_client_command() else {
        open::that(address)?;
        return Ok(());
    };

    let raw_cmd = shell_words::split(cmd)?;
    let Some(name) = raw_cmd.first() else {
        return Err(Error::invalid_value(
            "Missing application in command for web client.",
        ));
    };

    let mut cmd = Command::new(name);
    for arg in &raw_cmd[1..] {
        cmd.arg(arg.replace("${ADDRESS}", &address));
    }
    cmd.stderr(Stdio::null());
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.spawn()?;
    Ok(())
}
