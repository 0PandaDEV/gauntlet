use std::path::Path;

use clap::Parser;

use crate::client::start_client;
use crate::server::start_server;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    Open,
    Server,
    Download,
}

pub fn init() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Open => start_client(),
        Commands::Server => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            runtime.block_on(async {
                start_server().await
            }).unwrap();
        }
        Commands::Download => download_repo().unwrap()
    };
}

fn download_repo() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;

    let url = gix::url::parse(gix::path::os_str_into_bstr("file:///home/exidex/CLionProjects/testrepo".as_ref())?)?;
    let mut prepare_fetch = gix::clone::PrepareFetch::new(url, &temp_dir, gix::create::Kind::WithWorktree, Default::default(), Default::default())
        .unwrap()
        .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(1.try_into().unwrap()))
        .configure_remote(|mut remote| {
            remote.replace_refspecs(
                Some("+refs/heads/placeholdername/releases:refs/remotes/origin/placeholdername/releases"),
                gix::remote::Direction::Fetch,
            )?;

            Ok(remote)
        });

    let (mut prepare_checkout, _) = prepare_fetch.fetch_then_checkout(
        gix::progress::Discard,
        &gix::interrupt::IS_INTERRUPTED,
    )?;

    let (_repo, _) = prepare_checkout
        .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;


    let git_repo_dir = temp_dir.path();

    let plugins_path = git_repo_dir.join("plugins");
    let _version_path = plugins_path.join("v1");

    // copy_dir_all(version_path, )

    Ok(())
}


fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}