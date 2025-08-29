use std::{thread, time::Duration};

use crate::{
    cli::run_detached,
    core::{AnyControlMsg, Result, config::Config, server::client::Client},
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

    open::that(format!(
        "http://{}:{}/app",
        conf.server_address(),
        conf.port()
    ))?;
    Ok(())
}
