extern crate derive_more;

use clap::CommandFactory;
use clap::Parser;
use clap_verbosity_flag::{Level, Verbosity};
use golem_cli::command::profile::UniversalProfileAdd;
use golem_cli::config::{Config, Profile};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use golem_cli::init::{CliKind, DummyProfileAuth, GolemInitCommand, PrintCompletion};
use golem_cli::oss;
use golem_cli::oss::command::GolemOssCommand;
use golem_cloud_cli::cloud;
use golem_cloud_cli::cloud::command::GolemCloudCommand;
use golem_cloud_cli::cloud::completion::PrintCloudUniversalCompletion;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let home = dirs::home_dir().unwrap();
    let default_conf_dir = home.join(".golem");
    let config_dir = std::env::var("GOLEM_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or(default_conf_dir);

    if let Some(p) = Config::get_active_profile(CliKind::Universal, &config_dir) {
        let name = p.name.clone();

        match p.profile {
            Profile::Golem(p) => {
                let command = GolemOssCommand::<UniversalProfileAdd>::parse();

                init_tracing(&command.verbosity);
                info!("Golem CLI with profile: {}", name);

                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(oss::main::async_main(
                        command,
                        p,
                        CliKind::Universal,
                        config_dir,
                        Box::new(PrintOssUniversalCompletion()),
                        Box::new(DummyProfileAuth {}),
                    ))
            }
            Profile::GolemCloud(p) => {
                let command = GolemCloudCommand::<UniversalProfileAdd>::parse();

                init_tracing(&command.verbosity);
                info!("Golem Cloud CLI with profile: {}", name);

                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(cloud::main::async_main(
                        command,
                        name,
                        p,
                        CliKind::Universal,
                        config_dir,
                        Box::new(PrintCloudUniversalCompletion()),
                    ))
            }
        }
    } else {
        let command = GolemInitCommand::<UniversalProfileAdd>::parse();

        init_tracing(&command.verbosity);
        info!("Golem Init CLI");

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(golem_cli::init::async_main(
                command,
                CliKind::Universal,
                config_dir,
                Box::new(DummyProfileAuth {}),
                Box::new(PrintCloudUniversalCompletion()),
            ))
    }
}

fn init_tracing(verbosity: &Verbosity) {
    if let Some(level) = verbosity.log_level() {
        let tracing_level = match level {
            Level::Error => tracing::Level::ERROR,
            Level::Warn => tracing::Level::WARN,
            Level::Info => tracing::Level::INFO,
            Level::Debug => tracing::Level::DEBUG,
            Level::Trace => tracing::Level::TRACE,
        };

        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing_level)
            .with_writer(std::io::stderr)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }
}

struct PrintOssUniversalCompletion();

impl PrintCompletion for PrintOssUniversalCompletion {
    fn print_completion(&self, generator: clap_complete::Shell) {
        let mut cmd = GolemOssCommand::<UniversalProfileAdd>::command();
        let cmd_name = cmd.get_name().to_string();
        info!("Golem CLI - generating completion file for {generator:?}...");
        clap_complete::generate(generator, &mut cmd, cmd_name, &mut std::io::stdout());
    }
}
