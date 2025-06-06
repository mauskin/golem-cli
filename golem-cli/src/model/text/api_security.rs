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

use crate::model::text::fmt::*;
use crate::model::ApiSecurityScheme;
use cli_table::Table;
use golem_client::model::SecuritySchemeData;
use indoc::printdoc;

impl TextView for ApiSecurityScheme {
    fn log(&self) {
        printdoc!(
                    "
                    API Security Scheme: ID: {}, scopes: {}, client ID: {}, client secret: {}, redirect URL: {}
                    ",
                    format_message_highlight(&self.scheme_identifier),
                    &self.scopes.join(", "),
                    format_message_highlight(&self.client_id),
                    format_message_highlight(&self.client_secret),
                    format_message_highlight(&self.redirect_url),
                );
    }
}

#[derive(Table)]
struct ApiSecuritySchemeTableView {
    #[table(title = "ID")]
    pub id: String,
    #[table(title = "Provider")]
    pub provider: String,
    #[table(title = "Client ID")]
    pub client_id: String,
    #[table(title = "Client Secret")]
    pub client_secret: String,
    #[table(title = "Redirect URL")]
    pub redirect_url: String,
}

impl From<&SecuritySchemeData> for ApiSecuritySchemeTableView {
    fn from(value: &SecuritySchemeData) -> Self {
        Self {
            id: value.scheme_identifier.clone(),
            provider: value.provider_type.to_string(),
            client_id: value.client_id.clone(),
            client_secret: value.client_secret.clone(),
            redirect_url: value.redirect_url.clone(),
        }
    }
}
