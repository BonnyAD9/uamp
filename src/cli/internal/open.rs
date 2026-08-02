use std::path::{Path, PathBuf};

use itertools::Itertools;
use pareg::{ArgInto, ParegRef};

use crate::{
    background_app::run_background_app,
    cli::Run,
    core::{
        DataControlMsg, Error, Result, config::Config, server::client::Client,
    },
    env::RunType,
};

#[derive(Default, Debug)]
pub struct Open {
    files: Vec<PathBuf>,
}

impl Open {
    pub fn parse<'a, S: ArgInto<'a>>(
        pareg: &mut ParegRef<'a, S>,
    ) -> Result<Self> {
        let files = pareg
            .remaining()
            .iter()
            .map(|p| Result::Ok(p.arg_into::<&Path>()?.canonicalize()?))
            .try_collect::<_, _, Error>()?;
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
                    return Result::<_, Error>::Ok(());
                };

                client.send_ctrl(&[msg.take().unwrap()]).await?;
                Ok(())
            })?
        };

        let Some(msg) = msg else { return Ok(()) };

        run_background_app(conf, vec![msg])
    }
}
