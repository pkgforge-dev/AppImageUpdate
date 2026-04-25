use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use appimageupdate::config;
use appimageupdate::util::format_size;
use appimageupdate::{Error, Updater};
use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

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

    #[arg(short = 'l', long)]
    list_releases: bool,

    #[arg(short = 't', long, value_name = "TAG")]
    target_tag: Option<String>,

    #[arg(
        long,
        value_name = "URL",
        env = "GITHUB_API_PROXY",
        value_delimiter = ','
    )]
    github_api_proxy: Vec<String>,

    #[arg(
        long,
        value_name = "URL",
        env = "GITLAB_API_PROXY",
        value_delimiter = ','
    )]
    gitlab_api_proxy: Vec<String>,

    #[arg(
        long,
        value_name = "URL",
        env = "CODEBERG_API_PROXY",
        value_delimiter = ','
    )]
    codeberg_api_proxy: Vec<String>,

    #[arg(short = 'J', long, value_name = "N", default_value = "0")]
    jobs: usize,
}

#[derive(Debug, Default)]
struct Totals {
    downloaded: u64,
    saved: u64,
    updated: usize,
    copied: usize,
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
        config::set_github_proxies(cli.github_api_proxy.clone());
    }
    if !cli.gitlab_api_proxy.is_empty() {
        config::set_gitlab_proxies(cli.gitlab_api_proxy.clone());
    }
    if !cli.codeberg_api_proxy.is_empty() {
        config::set_codeberg_proxies(cli.codeberg_api_proxy.clone());
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

    let num_jobs = if cli.jobs == 0 {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1)
    } else {
        cli.jobs
    };
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_jobs)
        .build()
        .map_err(|e| Error::AppImage(format!("Failed to create thread pool: {}", e)))?;

    let multi = Arc::new(MultiProgress::new());

    if cli.check_for_update {
        let any_update_available = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        pool.install(|| {
            appimages.par_iter().for_each(|path| {
                let name = truncate_filename(path);
                match create_updater(&cli, path).and_then(|u| u.check_for_update()) {
                    Ok(has_update) => {
                        let msg = if has_update {
                            any_update_available.store(true, std::sync::atomic::Ordering::Relaxed);
                            "Update available"
                        } else {
                            "Up to date"
                        };
                        multi.suspend(|| eprintln!("  {} {}", name.trim(), msg));
                    }
                    Err(e) => {
                        multi.suspend(|| eprintln!("  {} Error: {}", name.trim(), e));
                        errors.lock().unwrap().push(format!(
                            "{}: {}",
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown"),
                            e
                        ));
                    }
                }
            });
        });

        for err in Arc::try_unwrap(errors).unwrap().into_inner().unwrap() {
            eprintln!("  {}", err);
        }

        if any_update_available.load(std::sync::atomic::Ordering::Relaxed) {
            std::process::exit(1);
        }
        return Ok(());
    }

    if cli.describe {
        for (i, path) in appimages.iter().enumerate() {
            if i > 0 {
                eprintln!();
            }
            describe_appimage(&cli, path);
        }
        return Ok(());
    }

    if cli.list_releases {
        for path in &appimages {
            let updater = create_updater(&cli, path)?;
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            match updater.list_releases() {
                Ok(releases) => {
                    if appimages.len() > 1 {
                        eprintln!("{}:", name);
                    }
                    let max_tag = releases.iter().map(|r| r.tag().len()).max().unwrap_or(0);
                    for r in &releases {
                        let date = r.published_at().split('T').next().unwrap_or("");
                        let pre = if r.is_prerelease() {
                            "  [pre-release]"
                        } else {
                            ""
                        };
                        eprintln!("  {:<width$}  {}{}", r.tag(), date, pre, width = max_tag);
                    }
                }
                Err(e) => eprintln!("  {} Error: {}", name, e),
            }
        }
        return Ok(());
    }

    let totals: Arc<Mutex<Totals>> = Arc::new(Mutex::new(Totals::default()));
    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    multi.suspend(|| eprintln!("Resolving update URLs..."));
    let mut url_mutexes: HashMap<String, Arc<Mutex<Option<PathBuf>>>> = HashMap::new();
    #[allow(clippy::type_complexity)]
    let mut tasks: Vec<(Updater, Option<Arc<Mutex<Option<PathBuf>>>>)> = Vec::new();

    for path in &appimages {
        let updater = create_updater(&cli, path)?;
        let url_mutex = updater
            .zsync_url()
            .ok()
            .filter(|u| !u.is_empty())
            .map(|url| {
                url_mutexes
                    .entry(url)
                    .or_insert_with(|| Arc::new(Mutex::new(None)))
                    .clone()
            });
        tasks.push((updater, url_mutex));
    }

    pool.install(|| {
        tasks.into_par_iter().for_each(|(updater, url_mutex)| {
            let source_path = updater.source_path().to_path_buf();
            match handle_appimage(&cli, updater, &multi, url_mutex) {
                Ok(result) => {
                    let mut t = totals.lock().unwrap();
                    t.downloaded += result.downloaded;
                    t.saved += result.saved;
                    if result.copied {
                        t.copied += 1;
                    } else if result.downloaded > 0 {
                        t.updated += 1;
                    }
                }
                Err(e) => errors.lock().unwrap().push(format!(
                    "{}: {}",
                    source_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown"),
                    e
                )),
            }
        });
    });

    let totals = Arc::try_unwrap(totals).unwrap().into_inner().unwrap();
    let errors = Arc::try_unwrap(errors).unwrap().into_inner().unwrap();

    if totals.updated + totals.copied > 0 {
        let mut parts = Vec::new();
        if totals.updated > 0 {
            parts.push(format!("{} updated", totals.updated));
        }
        if totals.copied > 0 {
            parts.push(format!("{} copied", totals.copied));
        }
        parts.push(format!(
            "↓ {} total, saved {}",
            format_size(totals.downloaded),
            format_size(totals.saved),
        ));
        eprintln!("{}", parts.join(", "));
    }

    if !errors.is_empty() {
        eprintln!("\nFailed to update {} AppImage(s):", errors.len());
        for err in &errors {
            eprintln!("  {}", err);
        }
        std::process::exit(1);
    }

    Ok(())
}

fn download_style() -> ProgressStyle {
    ProgressStyle::with_template("{prefix:.cyan} [{bar:20.cyan/blue}] {msg:.dim}")
        .expect("invalid style")
        .progress_chars("━━╾─")
}

fn create_bar() -> ProgressBar {
    let pb = ProgressBar::new(1);
    pb.set_style(download_style());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

fn truncate_filename(path: &Path) -> String {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    if filename.len() > 25 {
        format!("{}...", &filename[..22])
    } else {
        filename.to_string()
    }
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

fn is_appimage(path: &Path) -> bool {
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

fn describe_appimage(cli: &Cli, path: &Path) {
    println!("{}:", path.display());

    let updater = match create_updater(cli, path) {
        Ok(u) => u,
        Err(e) => {
            println!("  Error: {}", e);
            return;
        }
    };

    println!("  AppImage type: {}", updater.appimage_type());

    let info = updater.update_info_struct();
    println!("  Update info: {}", info.raw());
    println!(
        "  Update info type: {} ({})",
        info.type_display_name(),
        info.type_label()
    );

    if let Some(g) = info.generic_info() {
        println!("  ZSync URL: {}", g.zsync_url());
    } else if let Some(f) = info.forge_info() {
        if let appimageupdate::ForgeKind::Gitea { instance } = &f.kind {
            println!("  Instance: {}", instance);
        }
        println!("  Project: {}/{}", f.owner, f.repo);
        println!("  Tag: {}", f.tag);
        println!("  Filename: {}", f.filename);
    }

    match updater.zsync_url() {
        Ok(url) => println!("  Assembled URL: {}", url),
        Err(e) => println!("  Assembled URL: <unresolved: {}>", e),
    }
}

fn create_updater(cli: &Cli, path: &Path) -> Result<Updater, Error> {
    let updater = if let Some(ref info) = cli.update_info {
        Updater::with_update_info(path, info)?
    } else {
        Updater::new(path)?
    };

    if let Some(ref tag) = cli.target_tag {
        updater.target_tag(tag)
    } else {
        Ok(updater)
    }
}

fn handle_appimage(
    cli: &Cli,
    updater: Updater,
    multi: &MultiProgress,
    url_mutex: Option<Arc<Mutex<Option<PathBuf>>>>,
) -> Result<UpdateResult, Error> {
    let path = updater.source_path().to_path_buf();
    let name = truncate_filename(&path);
    let full_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut updater = updater;
    if let Some(output_dir) = config::get_output_dir(cli.output_dir.clone()) {
        updater = updater.output_dir(&output_dir);
    }
    if cli.overwrite {
        updater = updater.overwrite(true);
    }

    let source_path = updater.source_path().to_path_buf();
    let (target_path, _target_size) = updater.target_info()?;

    if !updater.check_for_update()? {
        multi.suspend(|| eprintln!("  {} Up to date", full_name));
        return Ok(UpdateResult::default());
    }

    if let Some(ref url_mutex) = url_mutex {
        let mut guard = url_mutex.lock().unwrap();

        if let Some(ref downloaded_path) = *guard {
            if target_path.exists() {
                multi.suspend(|| eprintln!("  {} Up to date", full_name));
                return Ok(UpdateResult::default());
            }

            let perms = fs::metadata(&source_path).ok().map(|m| m.permissions());
            fs::copy(downloaded_path, &target_path)?;
            if let Some(perms) = perms {
                fs::set_permissions(&target_path, perms)?;
            }

            if config::get_remove_old(if cli.remove_old { Some(true) } else { None })
                && target_path != source_path
            {
                fs::remove_file(&source_path)?;
            }

            multi.suspend(|| eprintln!("  {} Updated (copied)", full_name));
            return Ok(UpdateResult {
                downloaded: 0,
                saved: 0,
                copied: true,
            });
        }

        let (new_path, stats) = run_download(multi, &name, &full_name, updater)?;

        *guard = Some(new_path.clone());
        drop(guard);
        do_remove_old(cli, &source_path, &new_path, &stats);

        return Ok(UpdateResult {
            downloaded: stats.bytes_downloaded(),
            saved: stats.bytes_reused(),
            copied: false,
        });
    }

    let (new_path, stats) = run_download(multi, &name, &full_name, updater)?;
    do_remove_old(cli, &source_path, &new_path, &stats);

    Ok(UpdateResult {
        downloaded: stats.bytes_downloaded(),
        saved: stats.bytes_reused(),
        copied: false,
    })
}

fn run_download(
    multi: &MultiProgress,
    name: &str,
    full_name: &str,
    mut updater: Updater,
) -> Result<(PathBuf, appimageupdate::UpdateStats), Error> {
    let pb = multi.add(create_bar());
    pb.set_prefix(name.to_string());
    pb.set_length(100);
    pb.set_position(0);
    pb.set_message("Scanning blocks...");

    let pb_clone = pb.clone();
    updater = updater.progress_callback(move |done, total| {
        if total > 0 {
            pb_clone.set_length(total);
            pb_clone.set_position(done);
            pb_clone.set_message(format!(
                "{:>3}% ↓ {}/{}",
                done * 100 / total,
                format_size(done),
                format_size(total)
            ));
        }
    });

    let (new_path, stats) = updater.perform_update()?;

    pb.finish_and_clear();
    multi.suspend(|| {
        eprintln!(
            "  {} ↓ {} | saved {}% ({})",
            full_name.trim(),
            format_size(stats.bytes_downloaded()),
            stats.saved_percentage(),
            format_size(stats.bytes_reused())
        );
    });

    Ok((new_path, stats))
}

fn do_remove_old(
    cli: &Cli,
    source_path: &Path,
    new_path: &Path,
    stats: &appimageupdate::UpdateStats,
) {
    if !config::get_remove_old(if cli.remove_old { Some(true) } else { None }) {
        return;
    }
    if let Some(ref backup) = stats.backup_path {
        let _ = fs::remove_file(backup);
    } else if new_path != source_path {
        let _ = fs::remove_file(source_path);
    }
}

#[derive(Debug, Default)]
struct UpdateResult {
    downloaded: u64,
    saved: u64,
    copied: bool,
}
