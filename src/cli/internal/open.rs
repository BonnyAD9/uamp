use std::path::{Path, PathBuf};

use itertools::Itertools;
use pareg::Pareg;

use crate::{
    background_app::run_background_app,
    cli::Run,
    core::{DataControlMsg, Result, config::Config, server::client::Client},
    env::RunType,
};

#[derive(Default, Debug)]
pub struct Open {
    files: Vec<PathBuf>,
}

impl Open {
    pub fn parse(pareg: &mut Pareg) -> Result<Self> {
        let files = pareg
            .remaining()
            .iter()
            .map(|p| Path::new(&p).canonicalize())
            .try_collect()?;
        pareg.skip_all();
        Ok(Self { files })
    }

    pub fn act(self, conf: Config) -> Result<()> {
        if self.files.is_empty() {
            return Run {
                run_type: RunType::WebClient,
                ..Default::default()
            }
            .run_app(conf);
        }

        let address = format!("{}:{}", conf.server_address(), conf.port());
        let mut msg = Some(DataControlMsg::PlayTmp(self.files).into());
        {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            rt.block_on(async {
                let Ok(mut client) = Client::connect(address.clone()).await
                else {
                    return Result::Ok(());
                };

                client.send_ctrl(&[msg.take().unwrap()]).await?;
                Ok(())
            })?
        };

        let Some(msg) = msg else { return Ok(()) };

        run_background_app(conf, vec![msg])
    }
}
