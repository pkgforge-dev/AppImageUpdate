use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;

use appimageupdate::config;
use appimageupdate::{Error, Updater};

#[derive(Parser)]
#[command(name = "appimageupdate")]
#[command(about = "Update AppImages using delta updates", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Update {
        #[arg(value_name = "APPIMAGE")]
        path: PathBuf,

        #[arg(short, long)]
        overwrite: bool,

        #[arg(short, long, value_name = "DIR")]
        output: Option<PathBuf>,

        #[arg(long, value_name = "URL", env = "GITHUB_API_PROXY")]
        github_api_proxy: Option<String>,
    },

    Check {
        #[arg(value_name = "APPIMAGE")]
        path: PathBuf,

        #[arg(long, value_name = "URL", env = "GITHUB_API_PROXY")]
        github_api_proxy: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("\nError: {}", e);
        std::process::exit(1);
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn run(cli: Cli) -> Result<(), Error> {
    match cli.command {
        Commands::Update {
            path,
            overwrite,
            output,
            github_api_proxy,
        } => {
            config::init(github_api_proxy);

            let mut updater = Updater::new(&path)?;

            if overwrite {
                updater = updater.overwrite(true);
            }

            if let Some(dir) = output {
                updater = updater.output_dir(dir);
            }

            let source_path = updater.source_path();
            let source_size = updater.source_size();
            let (target_path, target_size) = updater.target_info()?;

            println!(
                "Source:   {} ({})",
                source_path.display(),
                format_size(source_size)
            );
            println!(
                "Target:   {} ({})",
                target_path.display(),
                format_size(target_size)
            );
            println!();

            if updater.check_for_update()? {
                println!("Performing delta update...");
                let (new_path, stats) = updater.perform_update()?;

                if stats.blocks_reused > 0 || stats.blocks_downloaded > 0 {
                    println!();
                    println!(
                        "Reused:      {:>10}  ({} blocks)",
                        format_size(stats.bytes_reused()),
                        stats.blocks_reused
                    );
                    println!(
                        "Downloaded:  {:>10}  ({} blocks)",
                        format_size(stats.bytes_downloaded()),
                        stats.blocks_downloaded
                    );
                    println!(
                        "Saved:       {:>10}  ({}%)",
                        format_size(stats.bytes_reused()),
                        stats.saved_percentage()
                    );
                }

                println!();
                println!("Updated: {}", new_path.display());
            } else {
                println!("Already up to date!");
            }
        }
        Commands::Check {
            path,
            github_api_proxy,
        } => {
            config::init(github_api_proxy);

            let updater = Updater::new(&path)?;

            let source_path = updater.source_path();
            let source_size = updater.source_size();
            let (target_path, target_size) = updater.target_info()?;

            println!(
                "Source:   {} ({})",
                source_path.display(),
                format_size(source_size)
            );
            println!(
                "Target:   {} ({})",
                target_path.display(),
                format_size(target_size)
            );
            println!();

            if updater.check_for_update()? {
                println!("Status: Update available");
            } else {
                println!("Status: Up to date");
            }
        }
    }

    Ok(())
}
