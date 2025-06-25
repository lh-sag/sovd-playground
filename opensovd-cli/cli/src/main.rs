use clap::Command;
use opensovd_tracing::info;

fn main() -> std::io::Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt().compact().init();

    const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), ' ', '(', env!("COMMIT_SHA"), ')');

    info!(version = VERSION, "OpenSOVD CLI client");

    let _matches = Command::new(env!("CARGO_BIN_NAME"))
        .about("OpenSOVD cli client")
        .version(VERSION)
        .get_matches();

    info!("OpenSOVD CLI client finished successfully");
    Ok(())
}
