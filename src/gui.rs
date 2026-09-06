//! Graphical frontend built on zenity-rs dialogs.
//!
//! Used when `--gui` is passed, or when the tool is launched without arguments
//! outside a terminal, such as by double-clicking it in a file manager.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use appimageupdate::util::format_size;
use appimageupdate::{Error, UpdateStats, config};
use zenity_rs::{ButtonPreset, DialogResult, FileFilter, FileSelectResult, Icon, ProgressHandle};

use crate::{Cli, collect_appimages, create_updater, do_remove_old};

const TITLE: &str = "AppImageUpdate";

#[derive(Default)]
struct Summary {
    updated: usize,
    up_to_date: usize,
    downloaded: u64,
    saved: u64,
    cancelled: bool,
    errors: Vec<String>,
}

/// Runs the graphical update flow, asking for AppImages when none were given.
pub fn run(cli: &Cli) -> Result<(), Error> {
    let picked;
    let paths = if cli.paths.is_empty() {
        match pick_appimages()? {
            Some(paths) => {
                picked = paths;
                &picked
            }
            None => return Ok(()),
        }
    } else {
        &cli.paths
    };

    let appimages = collect_appimages(paths)?;
    if appimages.is_empty() {
        return show_message("No AppImages found.", Icon::Warning);
    }

    let (handle, dialog) = zenity_rs::progress()
        .title(TITLE)
        .text("Checking for updates...")
        .width(420)
        // Names end in the version and architecture, so keep the tail.
        .ellipsize_middle(true)
        .auto_close(true)
        .spawn()
        .map_err(dialog_error)?;

    let summary = update_all(cli, &appimages, &handle);
    handle.finish();
    dialog.join().map_err(dialog_error)?;

    report(&summary)
}

/// Asks whether to update chosen AppImages or everything in a folder, then
/// runs the matching picker. `None` means the user backed out.
fn pick_appimages() -> Result<Option<Vec<PathBuf>>, Error> {
    let choice = zenity_rs::message()
        .title(TITLE)
        .text("What do you want to update?")
        .icon(Icon::Question)
        .buttons(ButtonPreset::Custom(vec![
            "AppImages".to_string(),
            "Folder".to_string(),
            "Cancel".to_string(),
        ]))
        .show()
        .map_err(dialog_error)?;

    match choice {
        DialogResult::Button(0) => pick_files(),
        DialogResult::Button(1) => pick_folder(),
        _ => Ok(None),
    }
}

fn pick_files() -> Result<Option<Vec<PathBuf>>, Error> {
    let mut picker = zenity_rs::file_select()
        .title("Select AppImages to update")
        .multiple(true)
        .add_filter(FileFilter {
            name: "AppImages".to_string(),
            patterns: vec!["*.AppImage".to_string()],
        })
        .add_filter(FileFilter {
            name: "All files".to_string(),
            patterns: vec!["*".to_string()],
        });

    if let Some(dir) = start_dir() {
        picker = picker.start_path(&dir);
    }

    Ok(selection(picker.show().map_err(dialog_error)?))
}

fn pick_folder() -> Result<Option<Vec<PathBuf>>, Error> {
    let mut picker = zenity_rs::file_select()
        .title("Select a folder of AppImages to update")
        .directory(true);

    if let Some(dir) = start_dir() {
        picker = picker.start_path(&dir);
    }

    Ok(selection(picker.show().map_err(dialog_error)?))
}

fn selection(result: FileSelectResult) -> Option<Vec<PathBuf>> {
    match result {
        FileSelectResult::Selected(path) => Some(vec![path]),
        FileSelectResult::SelectedMultiple(paths) => Some(paths),
        FileSelectResult::Cancelled | FileSelectResult::Closed | FileSelectResult::Timeout => None,
    }
}

fn start_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let applications = home.join("Applications");
    Some(if applications.is_dir() {
        applications
    } else {
        home
    })
}

fn update_all(cli: &Cli, appimages: &[PathBuf], handle: &ProgressHandle) -> Summary {
    let mut summary = Summary::default();
    let total = appimages.len() as u64;

    for (index, path) in appimages.iter().enumerate() {
        if handle.is_cancelled() {
            summary.cancelled = true;
            break;
        }

        let index = index as u64;
        let name = file_name(path);
        handle.set_percentage((index * 100 / total) as u32);
        handle.set_text(&format!("Checking {}", name));
        // The SHA1 check and the local block scan report nothing, so pulsate
        // until the first download progress arrives.
        handle.set_pulsating(true);

        match update_one(cli, path, index, total, handle) {
            Ok(Some(stats)) => {
                summary.updated += 1;
                summary.downloaded += stats.bytes_downloaded();
                summary.saved += stats.bytes_reused();
            }
            Ok(None) => summary.up_to_date += 1,
            Err(e) => summary.errors.push(format!("{}: {}", name, e)),
        }
    }

    handle.set_pulsating(false);
    handle.set_percentage(100);
    summary
}

/// Updates a single AppImage, returning `None` when it is already up to date.
fn update_one(
    cli: &Cli,
    path: &Path,
    index: u64,
    total: u64,
    handle: &ProgressHandle,
) -> Result<Option<UpdateStats>, Error> {
    let mut updater = create_updater(cli, path)?;
    if let Some(output_dir) = config::get_output_dir(cli.output_dir.clone()) {
        updater = updater.output_dir(&output_dir);
    }
    if cli.overwrite {
        updater = updater.overwrite(true);
    }

    let source_path = updater.source_path().to_path_buf();
    if !updater.check_for_update()? {
        return Ok(None);
    }

    let name = file_name(path);
    handle.set_text(&format!("Scanning {}", name));

    let progress = handle.clone();
    let last_percentage = AtomicU32::new(u32::MAX);
    updater = updater.progress_callback(move |done, size| {
        if size == 0 {
            return;
        }
        let percentage = ((index * 100 + done * 100 / size) / total) as u32;
        let previous = last_percentage.swap(percentage, Ordering::Relaxed);
        if previous == percentage {
            return;
        }
        if previous == u32::MAX {
            progress.set_pulsating(false);
        }
        progress.set_percentage(percentage);
        progress.set_text(&format!(
            "{} - {} of {}",
            name,
            format_size(done),
            format_size(size)
        ));
    });

    let (new_path, stats) = updater.perform_update()?;
    do_remove_old(cli, &source_path, &new_path, &stats);

    Ok(Some(stats))
}

fn report(summary: &Summary) -> Result<(), Error> {
    if !summary.errors.is_empty() {
        let text = format!(
            "Failed to update {} AppImage(s):\n\n{}",
            summary.errors.len(),
            summary.errors.join("\n")
        );
        show_message(&text, Icon::Error)?;
        return Err(Error::AppImage(format!(
            "failed to update {} AppImage(s)",
            summary.errors.len()
        )));
    }

    let mut lines = Vec::new();
    if summary.cancelled {
        lines.push("Update cancelled.".to_string());
    }
    if summary.updated > 0 {
        lines.push(format!(
            "Updated {} AppImage(s), downloaded {} and reused {}.",
            summary.updated,
            format_size(summary.downloaded),
            format_size(summary.saved)
        ));
    }
    if summary.up_to_date > 0 {
        lines.push(format!(
            "{} AppImage(s) already up to date.",
            summary.up_to_date
        ));
    }
    if lines.is_empty() {
        lines.push("Nothing to do.".to_string());
    }

    let icon = if summary.cancelled {
        Icon::Warning
    } else {
        Icon::Info
    };
    show_message(&lines.join("\n"), icon)
}

fn show_message(text: &str, icon: Icon) -> Result<(), Error> {
    zenity_rs::message()
        .title(TITLE)
        .text(text)
        .icon(icon)
        .buttons(ButtonPreset::Ok)
        .show()
        .map(|_| ())
        .map_err(dialog_error)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn dialog_error(e: zenity_rs::Error) -> Error {
    Error::AppImage(format!("GUI unavailable: {}", e))
}
