use anyhow::anyhow;
use clap::{Args, Parser, Subcommand};
use url::Url;

#[derive(Parser, Debug)]
#[clap(
    name = "snap-issue-reproducer",
    about = "Reproduce the SNAP forwarding issue",
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Server(ServerOpt),
    Client(ClientOpt),
}

#[derive(Args, Debug)]
struct ServerOpt {
    /// Address to listen on
    #[clap(long)]
    listen: scion_proto::address::SocketAddr,

    /// Address of the endhost API to connect to for scion path resolution. Required when using SCION.
    #[clap(long = "endhost-api")]
    endhost_api_address: Url,

    /// Token for authentication with the endhost API
    #[clap(long = "snap-token")]
    snap_token: Option<String>,

    /// Tracing level (trace, debug, info, warn, error)
    #[clap(long = "log", default_value = "info")]
    log_level: tracing::Level,
}

#[derive(Args, Debug)]
struct ClientOpt {
    /// Address of server to connect to (e.g. [0-0,server.example.com]:4433)
    remote: scion_proto::address::SocketAddr,

    /// Address of the endhost API to connect to for scion path resolution. Required when using SCION.
    #[clap(long = "endhost-api")]
    endhost_api_address: Url,

    /// Token for authentication with the endhost API
    #[clap(long = "snap-token")]
    snap_token: Option<String>,

    /// Tracing level (trace, debug, info, warn, error)
    #[clap(long = "log", default_value = "info")]
    log_level: tracing::Level,
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Server(opt) => run_server(opt),
        Command::Client(opt) => run_client(opt),
    };
    if let Err(ref err) = result {
        tracing::error!(error = %err, "command failed");
    }
    result
}

#[tokio::main]
async fn run_server(opt: ServerOpt) -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(opt.log_level)
        .try_init()
        .map_err(|err| anyhow!("failed to init tracing: {err}"))?;
    let server = snap_issue_reproducer::server::Server::new(
        opt.listen,
        opt.endhost_api_address,
        opt.snap_token,
    );
    server.run().await?;
    Ok(())
}

#[tokio::main]
async fn run_client(opt: ClientOpt) -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(opt.log_level)
        .try_init()
        .map_err(|err| anyhow!("failed to init tracing: {err}"))?;
    let client = snap_issue_reproducer::client::Client::new(
        opt.remote,
        opt.endhost_api_address,
        opt.snap_token,
    );
    client.run().await?;
    Ok(())
}
