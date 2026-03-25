use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use appimageupdate::config;
use appimageupdate::util::format_size;
use appimageupdate::{Error, Updater};
use clap::Parser;

#[derive(Parser)]
#[command(name = "appimageupdate")]
#[command(about = "AppImage companion tool taking care of updates for the commandline.", long_about = None)]
#[command(version)]
struct Cli {
    #[arg(value_name = "APPIMAGE", num_args(1..))]
    paths: Vec<PathBuf>,

    #[arg(short = 'O', long)]
    overwrite: bool,

    #[arg(short = 'r', long)]
    remove_old: bool,

    #[arg(long, value_name = "DIR")]
    output_dir: Option<PathBuf>,

    #[arg(short = 'u', long, value_name = "INFO")]
    update_info: Option<String>,

    #[arg(short = 'd', long)]
    describe: bool,

    #[arg(short = 'j', long)]
    check_for_update: bool,

    #[arg(
        long,
        value_name = "URL",
        env = "GITHUB_API_PROXY",
        value_delimiter = ','
    )]
    github_api_proxy: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    config::init();

    if let Err(e) = run(cli) {
        eprintln!("\nError: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Error> {
    if !cli.github_api_proxy.is_empty() {
        config::set_proxies(cli.github_api_proxy.clone());
    }
    if cli.paths.is_empty() {
        return Err(Error::AppImage(
            "No AppImage path provided. Use --help for usage.".into(),
        ));
    }
    let appimages = collect_appimages(&cli.paths)?;
    if appimages.is_empty() {
        return Err(Error::AppImage("No AppImages found.".into()));
    }

    let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut ungrouped: Vec<PathBuf> = Vec::new();

    for path in &appimages {
        if let Ok(updater) = create_updater(&cli, path)
            && let Ok(zsync_url) = updater.zsync_url()
        {
            groups.entry(zsync_url).or_default().push(path.clone());
            continue;
        }
        ungrouped.push(path.clone());
    }

    let mut errors = Vec::new();
    let mut updated_files: HashMap<String, PathBuf> = HashMap::new();
    let mut any_update_available = false;

    for (zsync_url, paths) in &groups {
        if paths.len() > 1 && !cli.check_for_update {
            println!(
                "\n=== Group ({} AppImages, same update source) ===",
                paths.len()
            );
        }
        for path in paths {
            if appimages.len() > 1 && !cli.check_for_update {
                println!("\n=== {} ===", path.display());
            }
            if cli.check_for_update {
                print!("Checking: {} ... ", path.display());
                use std::io::Write;
                std::io::stdout().flush().ok();
                match check_update(&cli, path) {
                    Ok(has_update) => {
                        if has_update {
                            println!("Update available");
                            any_update_available = true;
                        } else {
                            println!("Up to date");
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        errors.push(path.clone());
                    }
                }
            } else if let Err(e) = handle_appimage(&cli, path, zsync_url, &mut updated_files) {
                eprintln!("Error updating {}: {}", path.display(), e);
                errors.push(path.clone());
            }
        }
    }

    for path in &ungrouped {
        if !cli.check_for_update {
            println!("\n=== {} ===", path.display());
        }
        if cli.check_for_update {
            print!("Checking: {} ... ", path.display());
            use std::io::Write;
            std::io::stdout().flush().ok();
            match check_update(&cli, path) {
                Ok(has_update) => {
                    if has_update {
                        println!("Update available");
                        any_update_available = true;
                    } else {
                        println!("Up to date");
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                    errors.push(path.clone());
                }
            }
        } else if let Err(e) = handle_appimage(&cli, path, "", &mut updated_files) {
            eprintln!("Error updating {}: {}", path.display(), e);
            errors.push(path.clone());
        }
    }

    if cli.check_for_update {
        std::process::exit(if any_update_available { 1 } else { 0 });
    }

    if !errors.is_empty() {
        eprintln!("\nFailed to update {} AppImage(s)", errors.len());
        std::process::exit(1);
    }
    Ok(())
}

fn collect_appimages(paths: &[PathBuf]) -> Result<Vec<PathBuf>, Error> {
    let mut appimages = Vec::new();
    for path in paths {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_file() && is_appimage(&entry_path) {
                    appimages.push(entry_path);
                }
            }
        } else if path.is_file() {
            appimages.push(path.clone());
        } else {
            return Err(Error::AppImage(format!(
                "Path does not exist: {}",
                path.display()
            )));
        }
    }
    appimages.sort();
    appimages.dedup();
    Ok(appimages)
}

fn is_appimage(path: &PathBuf) -> bool {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut magic = [0u8; 3];
    if file.seek(SeekFrom::Start(8)).is_err() {
        return false;
    }
    if file.read_exact(&mut magic).is_err() {
        return false;
    }
    &magic[0..2] == b"AI" && (magic[2] == 1 || magic[2] == 2)
}

fn create_updater(cli: &Cli, path: &PathBuf) -> Result<Updater, Error> {
    if let Some(ref update_info) = cli.update_info {
        Updater::with_update_info(path, update_info)
    } else {
        Updater::new(path)
    }
}

fn check_update(cli: &Cli, path: &PathBuf) -> Result<bool, Error> {
    let updater = create_updater(cli, path)?;
    updater.check_for_update()
}

fn handle_appimage(
    cli: &Cli,
    path: &PathBuf,
    zsync_url: &str,
    updated_files: &mut HashMap<String, PathBuf>,
) -> Result<(), Error> {
    let mut updater = create_updater(cli, path)?;
    if let Some(output_dir) = config::get_output_dir(cli.output_dir.clone()) {
        updater = updater.output_dir(&output_dir);
    }
    if cli.overwrite {
        updater = updater.overwrite(true);
    }

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

    if !updater.check_for_update()? {
        println!("Already up to date!");
        return Ok(());
    }

    if let Some(existing) = updated_files
        .get(zsync_url)
        .filter(|_| !zsync_url.is_empty())
    {
        if existing == &target_path {
            println!("Already updated (same target)");
        } else {
            println!("Copying from {}...", existing.display());
            let perms = fs::metadata(&source_path).ok().map(|m| m.permissions());
            fs::copy(existing, &target_path)?;
            if let Some(perms) = perms {
                fs::set_permissions(&target_path, perms)?;
            }
            println!("Updated: {}", target_path.display());
            let remove_old = config::get_remove_old(if cli.remove_old { Some(true) } else { None });
            if remove_old && target_path != source_path {
                fs::remove_file(&source_path)?;
                println!("Removed old AppImage");
            }
        }
        return Ok(());
    }

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

    if !zsync_url.is_empty() {
        updated_files.insert(zsync_url.to_string(), new_path.clone());
    }

    let remove_old = config::get_remove_old(if cli.remove_old { Some(true) } else { None });
    if remove_old {
        if let Some(ref backup) = stats.backup_path {
            fs::remove_file(backup)?;
            println!("Removed old AppImage");
        } else if new_path != source_path {
            fs::remove_file(&source_path)?;
            println!("Removed old AppImage");
        }
    }
    Ok(())
}
