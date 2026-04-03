use crate::aria2::{find_file_download_by_info_hash, parse_tell_status, rpc_call};
use indicatif::{ProgressBar, ProgressStyle};
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub enum InputType {
    HttpUrl(String),
    TorrentFile(PathBuf),
}

pub fn classify_input(s: &str) -> InputType {
    if s.to_lowercase().ends_with(".torrent") {
        InputType::TorrentFile(PathBuf::from(s))
    } else {
        InputType::HttpUrl(s.to_string())
    }
}

pub fn resolve_inputs(inputs: &[String]) -> Vec<InputType> {
    if inputs.len() == 1 {
        let path = PathBuf::from(&inputs[0]);
        if path.is_file() {
            if path.to_string_lossy().to_lowercase().ends_with(".torrent") {
                return vec![InputType::TorrentFile(path)];
            }
            return fs::read_to_string(&path)
                .unwrap_or_default()
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(|l| classify_input(&l))
                .collect();
        }
    }
    inputs.iter().map(|s| classify_input(s)).collect()
}

pub fn display_name(input: &InputType) -> String {
    match input {
        InputType::HttpUrl(url) => url.split('/').last().unwrap_or(url).to_string(),
        InputType::TorrentFile(p) => p
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    }
}

pub fn print_inputs(inputs: &[InputType], connections: u32, save_dir: &PathBuf) {
    let orange = "\x1b[1;38;5;214m";
    let reset = "\x1b[0m";
    println!("\n\u{2b61} Downloading {} file(s)...", inputs.len());
    for input in inputs {
        match input {
            InputType::HttpUrl(url) => {
                println!("{orange}\u{27ea} Address \u{27eb}{reset} {url}");
            }
            InputType::TorrentFile(p) => {
                println!("{orange}\u{27ea} Torrent \u{27eb}{reset} {}", p.display());
            }
        }
    }
    println!("{orange}\u{27ea} Connection \u{27eb}{reset} {connections}");
    println!(
        "{orange}\u{27ea} Save Path \u{27eb}{reset} {}\n",
        save_dir.display()
    );
}

pub fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GiB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MiB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.2} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

pub struct ActiveDownload {
    pub gid: String,
    pub label: String,
    pub pb: ProgressBar,
    pub last_completed: u64,
    pub last_time: Instant,
    pub done: bool,
    pub is_torrent: bool,
    pub torrent_path: Option<PathBuf>,
    pub info_hash: Option<String>,
    pub total_bytes: u64,
    pub start_time: Instant,
    pub metadata_done: bool,
}

impl ActiveDownload {
    pub fn new(
        gid: String,
        label: String,
        pb: ProgressBar,
        is_torrent: bool,
        torrent_path: Option<PathBuf>,
    ) -> Self {
        Self {
            gid,
            label,
            pb,
            last_completed: 0,
            last_time: Instant::now(),
            done: false,
            is_torrent,
            torrent_path,
            info_hash: None,
            total_bytes: 0,
            start_time: Instant::now(),
            metadata_done: !is_torrent,
        }
    }
}

pub fn cleanup_torrent_files(save_dir: &PathBuf, dl: &ActiveDownload) {
    if let Some(ref tp) = dl.torrent_path {
        let _ = fs::remove_file(tp);
    }
    let aria2_file = save_dir.join(format!("{}.aria2", dl.label));
    if aria2_file.exists() {
        let _ = fs::remove_file(&aria2_file);
    }
    if let Some(ref info_hash) = dl.info_hash {
        for h in &[info_hash.to_lowercase(), info_hash.to_uppercase()] {
            let torrent_file = save_dir.join(format!("{}.torrent", h));
            if torrent_file.exists() {
                let _ = fs::remove_file(&torrent_file);
            }
        }
    }
}

pub fn create_progress_style() -> ProgressStyle {
    ProgressStyle::with_template(
        "{spinner:.green} [{prefix}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} @ {binary_bytes_per_sec}",
    )
    .unwrap()
    .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn Write| {
        write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap();
    })
    .progress_chars("\u{2588}\u{2593}\u{2592}\u{2591}")
}

pub fn run_download_loop(
    downloads: &mut Vec<ActiveDownload>,
    aria2: &mut crate::aria2::Aria2Process,
    save_dir: &PathBuf,
    overall_start: Instant,
) {
    let metadata_timeout = Duration::from_secs(300);

    loop {
        if crate::SHUTDOWN.load(std::sync::atomic::Ordering::SeqCst) {
            eprintln!("\n\u{2717} Interrupted by user");
            aria2.kill();
            std::process::exit(130);
        }

        let mut all_done = true;

        for dl in downloads.iter_mut() {
            if dl.done {
                continue;
            }

            if dl.is_torrent && !dl.metadata_done {
                let elapsed = dl.start_time.elapsed();
                if elapsed > metadata_timeout {
                    dl.pb.finish_with_message("\u{2717} Metadata timeout");
                    dl.done = true;
                    aria2.kill();
                    eprintln!("\n\u{2717} Metadata resolution timed out for: {}", dl.label);
                    std::process::exit(1);
                }
            }

            let status_params = format!(r#"["{}"]"#, dl.gid);
            let Some(json) = rpc_call(&aria2.rpc_url, "aria2.tellStatus", &status_params) else {
                if dl.is_torrent && !dl.metadata_done {
                    dl.pb.set_message("waiting for file download to start...");
                    all_done = false;
                    continue;
                }
                all_done = false;
                continue;
            };

            if json.contains(r#""error""#) && !json.contains(r#""result""#) {
                if dl.is_torrent && !dl.metadata_done {
                    dl.pb.set_message("waiting for file download to start...");
                    all_done = false;
                    continue;
                }
                all_done = false;
                continue;
            }

            let Some(parsed) = parse_tell_status(&json) else {
                all_done = false;
                continue;
            };

            if dl.info_hash.is_none() {
                dl.info_hash = parsed.info_hash.clone();
            }

            if dl.is_torrent && !dl.metadata_done && parsed.has_metadata {
                dl.pb.set_message("fetching metadata...");
                all_done = false;
                continue;
            }

            if dl.is_torrent && !dl.metadata_done && !parsed.has_metadata && parsed.total_length > 0
            {
                dl.metadata_done = true;
                dl.pb.set_length(parsed.total_length);
                dl.total_bytes = parsed.total_length;
                dl.pb.set_message("");
            }

            match parsed.status.as_str() {
                "complete" | "waiting" | "seeding" | "used" => {
                    if parsed.status == "used" {
                        if let Some(file_gid) = parsed.followed_by {
                            dl.gid = file_gid;
                            dl.metadata_done = false;
                            dl.pb.set_message("resolving torrent files...");
                            all_done = false;
                            continue;
                        }
                    }
                    if dl.is_torrent && parsed.has_metadata {
                        if let Some(file_gid) = parsed.followed_by {
                            dl.gid = file_gid;
                            dl.metadata_done = false;
                            dl.pb.set_message("resolving torrent files...");
                            all_done = false;
                            continue;
                        }
                        if let Some(ref info_hash) = dl.info_hash {
                            if let Some(file_gid) =
                                find_file_download_by_info_hash(&aria2.rpc_url, info_hash)
                            {
                                dl.gid = file_gid;
                                dl.metadata_done = false;
                                dl.pb.set_message("resolving torrent files...");
                                all_done = false;
                                continue;
                            }
                        }
                        dl.pb.set_message("waiting for file download to start...");
                        all_done = false;
                        continue;
                    }

                    if dl.total_bytes > 0 {
                        dl.pb.set_position(dl.total_bytes);
                    }
                    dl.pb.finish_with_message("\u{2713} Complete!");
                    dl.done = true;

                    if dl.is_torrent {
                        let _ = rpc_call(
                            &aria2.rpc_url,
                            "aria2.removeDownloadResult",
                            &format!(r#"["{}"]"#, dl.gid),
                        );
                        cleanup_torrent_files(save_dir, dl);
                    }
                }
                "active" => {
                    if parsed.total_length > 0 && dl.total_bytes == 0 {
                        dl.total_bytes = parsed.total_length;
                        dl.pb.set_length(parsed.total_length);
                    }
                    dl.pb
                        .set_position(parsed.completed_length.min(parsed.total_length));

                    let now = Instant::now();
                    let elapsed = now.duration_since(dl.last_time).as_secs_f64();
                    if elapsed >= 0.5 && parsed.completed_length != dl.last_completed {
                        let speed = (parsed.completed_length - dl.last_completed) as f64 / elapsed;
                        dl.pb.set_message(format!("@ {:.1} KiB/s", speed / 1024.0));
                        dl.last_completed = parsed.completed_length;
                        dl.last_time = now;
                    }
                    all_done = false;
                }
                "paused" => {
                    all_done = false;
                }
                "error" => {
                    dl.pb.finish_with_message("\u{2717} Error");
                    dl.done = true;
                    eprintln!("\n\u{2717} Download failed: {}", dl.label);
                }
                "removed" => {
                    dl.pb.finish_with_message("\u{2717} Removed");
                    dl.done = true;
                }
                _ => {
                    all_done = false;
                }
            }
        }

        if all_done {
            aria2.kill();

            let elapsed = overall_start.elapsed();
            let total_bytes: u64 = downloads.iter().map(|d| d.total_bytes).sum();
            let completed = downloads.iter().filter(|d| d.done).count();
            let failed = downloads.len() - completed;

            let sep = "\u{2500}".repeat(60);
            println!("\n{}", sep);
            println!("\u{2713} All downloads complete!");
            println!();
            for dl in downloads.iter() {
                let size = if dl.total_bytes > 0 {
                    format_size(dl.total_bytes)
                } else {
                    "unknown".to_string()
                };
                let dur = dl.start_time.elapsed();
                println!("  {}  {}  ({:.1}s)", dl.label, size, dur.as_secs_f64());
            }
            println!();
            println!("  Total: {}", format_size(total_bytes));
            println!("  Completed: {}/{}", completed, downloads.len());
            if failed > 0 {
                println!("  Failed: {}", failed);
            }
            println!("  Time: {:.1}s", elapsed.as_secs_f64());
            println!("{}", sep);
            return;
        }

        if let Ok(Some(s)) = aria2.try_wait() {
            if !s.success() {
                aria2.kill();
                eprintln!("\n\u{2717} aria2c exited with error");
                std::process::exit(1);
            }
        }

        std::thread::sleep(Duration::from_millis(250));
    }
}
