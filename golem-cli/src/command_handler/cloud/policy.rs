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

use crate::command::cloud::project::policy::PolicySubcommand;
use crate::command_handler::Handlers;
use crate::context::Context;
use crate::error::service::AnyhowMapServiceError;
use crate::model::text::project::{ProjectPolicyGetView, ProjectPolicyNewView};
use crate::model::{ProjectAction, ProjectPolicyId};
use golem_cloud_client::api::ProjectPolicyClient;
use golem_cloud_client::model::{ProjectActions, ProjectPolicyData};
use std::sync::Arc;

pub struct CloudProjectPolicyCommandHandler {
    ctx: Arc<Context>,
}

impl CloudProjectPolicyCommandHandler {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub async fn handler_command(&self, subcommand: PolicySubcommand) -> anyhow::Result<()> {
        match subcommand {
            PolicySubcommand::New {
                policy_name,
                actions,
            } => self.cmd_new(policy_name, actions).await,
            PolicySubcommand::Get { policy_id } => self.cmd_get(policy_id).await,
        }
    }

    async fn cmd_new(
        &self,
        policy_name: String,
        actions: Vec<ProjectAction>,
    ) -> anyhow::Result<()> {
        let policy = self
            .ctx
            .golem_clients_cloud()
            .await?
            .project_policy
            .create_project_policy(&ProjectPolicyData {
                name: policy_name,
                project_actions: ProjectActions {
                    actions: actions.into_iter().map(|a| a.into()).collect(),
                },
            })
            .await
            .map_service_error()?;

        self.ctx
            .log_handler()
            .log_view(&ProjectPolicyNewView(policy));

        Ok(())
    }

    async fn cmd_get(&self, policy_id: ProjectPolicyId) -> anyhow::Result<()> {
        let policy = self
            .ctx
            .golem_clients_cloud()
            .await?
            .project_policy
            .get_project_policies(&policy_id.0)
            .await
            .map_service_error()?;

        self.ctx
            .log_handler()
            .log_view(&ProjectPolicyGetView(policy));

        Ok(())
    }
}
