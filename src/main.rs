mod aria2;
mod config;
mod download;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar};

use crate::aria2::Aria2Process;
use crate::config::Config;
use crate::download::{
    create_progress_style, display_name, print_inputs, resolve_inputs, run_download_loop,
    ActiveDownload, InputType,
};

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

#[derive(Parser, Debug)]
#[command(
    name = "dl-rs",
    about = "Aria2 download wrapper with configurable connections"
)]
struct Args {
    #[arg()]
    urls: Vec<String>,

    #[arg(short, long)]
    connections: Option<u32>,

    #[arg(short = 'o', long)]
    dir: Option<PathBuf>,
}

fn main() {
    ctrlc::set_handler(|| {
        SHUTDOWN.store(true, std::sync::atomic::Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl+C handler");

    Config::save_if_not_exists();
    let cfg = Config::load();
    let args = Args::parse();

    let inputs = resolve_inputs(&args.urls);
    if inputs.is_empty() {
        eprintln!("No URLs provided");
        std::process::exit(1);
    }

    let resolved = cfg.merge_with_args(args.connections, args.dir);
    let connections = resolved.connections;
    let save_dir = resolved.download_dir.clone();

    if !save_dir.exists() {
        std::fs::create_dir_all(&save_dir).unwrap_or_else(|e| {
            eprintln!("Failed to create directory {}: {}", save_dir.display(), e);
            std::process::exit(1);
        });
    }

    print_inputs(&inputs, connections, &save_dir);

    let has_torrent = inputs
        .iter()
        .any(|i| matches!(i, InputType::TorrentFile(_)));

    let mut aria2 = Aria2Process::spawn(connections, &save_dir, has_torrent);

    let mp = MultiProgress::new();
    let style = create_progress_style();

    let draw_target = indicatif::ProgressDrawTarget::stderr();
    mp.set_draw_target(draw_target);

    let mut downloads: Vec<ActiveDownload> = Vec::new();

    let is_multi = inputs.len() > 1;

    for (i, input) in inputs.iter().enumerate() {
        let label = display_name(input);
        let dir_opt = format!(r#""dir":"{}""#, save_dir.display());

        let add_result = match input {
            InputType::HttpUrl(url) => {
                let params = format!(r#"[["{}"], {{{}}}]"#, url, dir_opt);
                aria2::rpc_call(&aria2.rpc_url, "aria2.addUri", &params)
            }
            InputType::TorrentFile(path) => {
                let content = match std::fs::read(path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!(
                            "\u{2717} Failed to read torrent file {}: {}",
                            path.display(),
                            e
                        );
                        continue;
                    }
                };

                if content.is_empty() || content[0] != b'd' {
                    eprintln!("\u{2717} Invalid torrent file: {}", path.display());
                    continue;
                }

                let b64 =
                    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &content);
                let params = format!(r#"["{}", [], {{{}}}]"#, b64, dir_opt);
                aria2::rpc_call(&aria2.rpc_url, "aria2.addTorrent", &params)
            }
        };

        let Some(gid) = add_result.as_deref().and_then(|r| {
            serde_json::from_str::<serde_json::Value>(r)
                .ok()
                .and_then(|v| {
                    v.get("result")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
        }) else {
            eprintln!("\u{2717} Failed to add download: {}", label);
            continue;
        };

        let pb = mp.add(ProgressBar::new(0));
        pb.set_style(style.clone());
        pb.set_message(label.clone());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let is_torrent = matches!(input, InputType::TorrentFile(_));
        let torrent_path = match input {
            InputType::TorrentFile(p) => Some(p.clone()),
            _ => None,
        };

        downloads.push(ActiveDownload::new(
            gid,
            label,
            pb,
            is_torrent,
            torrent_path,
            i,
            is_multi,
        ));
    }

    if downloads.is_empty() {
        eprintln!("\u{2717} No downloads could be started");
        aria2.kill();
        std::process::exit(1);
    }

    let overall_start = std::time::Instant::now();
    run_download_loop(&mut downloads, &mut aria2, &save_dir, overall_start);
}
