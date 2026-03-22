use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;

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
    },

    Check {
        #[arg(value_name = "APPIMAGE")]
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Error> {
    match cli.command {
        Commands::Update {
            path,
            overwrite,
            output,
        } => {
            let mut updater = Updater::new(&path)?;

            if overwrite {
                updater = updater.overwrite(true);
            }

            if let Some(dir) = output {
                updater = updater.output_dir(dir);
            }

            println!("Checking for update...");
            if updater.check_for_update()? {
                println!("Update available. Performing delta update...");
                let new_path = updater.perform_update()?;
                println!("Updated AppImage: {}", new_path.display());
            } else {
                let output_path = updater.output_path()?;
                println!("Already up to date: {}", output_path.display());
            }
        }
        Commands::Check { path } => {
            let updater = Updater::new(&path)?;

            if updater.check_for_update()? {
                println!("Update available.");
            } else {
                println!("No update available.");
            }
        }
    }

    Ok(())
}
