use std::path::PathBuf;

use clap::Parser;

use appimageupdate::config;
use appimageupdate::{Error, Updater};

#[derive(Parser)]
#[command(name = "appimageupdate")]
#[command(about = "AppImage companion tool taking care of updates for the commandline.", long_about = None)]
#[command(version)]
struct Cli {
    #[arg(value_name = "APPIMAGE")]
    path: Option<PathBuf>,

    #[arg(short = 'O', long)]
    overwrite: bool,

    #[arg(short = 'r', long)]
    remove_old: bool,

    #[arg(short = 'u', long, value_name = "INFO")]
    update_info: Option<String>,

    #[arg(short = 'd', long)]
    describe: bool,

    #[arg(short = 'j', long)]
    check_for_update: bool,

    #[arg(long, value_name = "URL", env = "GITHUB_API_PROXY")]
    github_api_proxy: Option<String>,
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
    config::init(cli.github_api_proxy);

    let path = cli.path.ok_or_else(|| {
        Error::AppImage("No AppImage path provided. Use --help for usage.".into())
    })?;

    let updater = if let Some(ref update_info) = cli.update_info {
        Updater::with_update_info(&path, update_info)?
    } else {
        Updater::new(&path)?
    };

    if cli.describe {
        let source_path = updater.source_path();
        let source_size = updater.source_size();
        let (target_path, target_size) = updater.target_info()?;

        println!("Path:         {}", source_path.display());
        println!("Size:         {}", format_size(source_size));
        println!("Target:       {}", target_path.display());
        println!("Target Size:  {}", format_size(target_size));
        println!("Update Info:  {}", updater.update_info());

        return Ok(());
    }

    if cli.check_for_update {
        let has_update = updater.check_for_update()?;
        std::process::exit(if has_update { 1 } else { 0 });
    }

    let source_path = updater.source_path().to_path_buf();
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

    let mut updater = updater;
    if cli.overwrite {
        updater = updater.overwrite(true);
    }

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

        if cli.remove_old && new_path != source_path {
            std::fs::remove_file(source_path)?;
            println!("Removed old AppImage");
        }
    } else {
        println!("Already up to date!");
    }

    Ok(())
}
