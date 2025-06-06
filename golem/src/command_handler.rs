// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::launch::{launch_golem_services, LaunchArgs};
use anyhow::anyhow;
use clap_verbosity_flag::Verbosity;
use golem_cli::command::server::ServerSubcommand;
use golem_cli::command_handler::CommandHandlerHooks;
use golem_cli::context::Context;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ServerCommandHandler;

impl CommandHandlerHooks for ServerCommandHandler {
    async fn handler_server_commands(
        &self,
        _ctx: Arc<Context>,
        subcommand: ServerSubcommand,
    ) -> anyhow::Result<()> {
        match subcommand {
            ServerSubcommand::Run {
                router_addr,
                router_port,
                custom_request_port,
                data_dir,
                clean,
            } => {
                let data_dir = match data_dir {
                    Some(data_dir) => data_dir,
                    None => default_data_dir()?,
                };
                if clean && tokio::fs::metadata(&data_dir).await.is_ok() {
                    clean_data_dir(&data_dir).await?;
                };

                launch_golem_services(&LaunchArgs {
                    router_addr,
                    router_port,
                    custom_request_port,
                    data_dir,
                })
                .await?;

                Ok(())
            }
            ServerSubcommand::Clean => clean_data_dir(&default_data_dir()?).await,
        }
    }

    fn override_verbosity(verbosity: Verbosity) -> Verbosity {
        if verbosity.is_present() {
            verbosity
        } else {
            Verbosity::new(2, 0)
        }
    }

    fn override_pretty_mode() -> bool {
        true
    }
}

fn default_data_dir() -> anyhow::Result<PathBuf> {
    Ok(dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Failed to get data local dir"))?
        .join("golem"))
}

async fn clean_data_dir(data_dir: &Path) -> anyhow::Result<()> {
    tokio::fs::remove_dir_all(&data_dir)
        .await
        .map_err(|err| anyhow!("Failed cleaning data dir ({}): {}", data_dir.display(), err))
}
