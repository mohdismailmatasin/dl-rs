use std::process::{Child, Command, Stdio};
use std::time::Duration;

use serde_json::Value;

pub struct Aria2Process {
    pub child: Child,
    pub rpc_url: String,
}

impl Aria2Process {
    pub fn spawn(_connections: u32, _save_dir: &std::path::Path, has_torrent: bool) -> Self {
        let rpc_port = 6800 + (std::process::id() % 1000) as u16;
        let bt_port = 50000 + (std::process::id() % 10000) as u16;
        let rpc_url = format!("http://localhost:{}/jsonrpc", rpc_port);

        let mut aria_cmd = Command::new("aria2c");
        aria_cmd
            .arg("-x")
            .arg(_connections.to_string())
            .arg("-s")
            .arg(_connections.to_string())
            .arg("-d")
            .arg(_save_dir)
            .arg("--enable-rpc")
            .arg(format!("--rpc-listen-port={}", rpc_port))
            .arg("--rpc-listen-all=false")
            .arg("--no-conf")
            .arg("--min-split-size=1M")
            .arg("--file-allocation=none")
            .arg("--connect-timeout=10")
            .arg("--timeout=60")
            .arg("--max-tries=5")
            .arg("--retry-wait=3")
            .arg("--continue=true")
            .arg("--stream-piece-selector=default")
            .arg("--lowest-speed-limit=0")
            .arg("--max-file-not-found=5")
            .arg("--http-accept-gzip=true")
            .arg("--max-overall-download-limit=0")
            .arg("--max-download-limit=0")
            .arg("--summary-interval=0")
            .arg("--async-dns=true")
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if has_torrent {
            aria_cmd
                .arg("--enable-dht")
                .arg("--enable-dht6")
                .arg("--enable-peer-exchange")
                .arg("--follow-torrent=true")
                .arg(format!("--listen-port={}", bt_port))
                .arg("--seed-time=0")
                .arg("--bt-remove-unselected-file=true")
                .arg("--bt-save-metadata=true");
        }

        let mut child = aria_cmd
            .spawn()
            .expect("Failed to start aria2c. Is it installed?");

        drop(child.stdout.take());
        drop(child.stderr.take());

        for _i in 0..20 {
            std::thread::sleep(Duration::from_millis(100));
            if ureq::post(&rpc_url)
                .set("Content-Type", "application/json")
                .timeout(Duration::from_secs(1))
                .send_string(
                    r#"{"jsonrpc":"2.0","method":"aria2.getVersion","id":"q","params":[]}"#,
                )
                .is_ok()
            {
                break;
            }
        }

        Self { child, rpc_url }
    }

    pub fn kill(&mut self) {
        let _ = self.child.kill();
    }

    pub fn try_wait(&mut self) -> Result<Option<std::process::ExitStatus>, std::io::Error> {
        self.child.try_wait()
    }
}

pub fn rpc_call(rpc_url: &str, method: &str, params: &str) -> Option<String> {
    let body = format!(
        r#"{{"jsonrpc":"2.0","method":"{}","id":"q","params":{}}}"#,
        method, params
    );

    let resp = ureq::post(rpc_url)
        .set("Content-Type", "application/json")
        .timeout(Duration::from_secs(5))
        .send_string(&body)
        .ok()?;

    resp.into_string().ok()
}

pub fn parse_tell_status(json: &str) -> Option<Aria2Status> {
    let val: Value = serde_json::from_str(json).ok()?;
    let result = val.get("result")?;
    Some(Aria2Status {
        status: result
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        completed_length: result
            .get("completedLength")
            .and_then(|v| v.as_str().and_then(|s| s.parse::<u64>().ok()))
            .or_else(|| result.get("completedLength").and_then(|v| v.as_u64()))
            .unwrap_or(0),
        total_length: result
            .get("totalLength")
            .and_then(|v| v.as_str().and_then(|s| s.parse::<u64>().ok()))
            .or_else(|| result.get("totalLength").and_then(|v| v.as_u64()))
            .unwrap_or(0),
        info_hash: result
            .get("infoHash")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        followed_by: result
            .get("followedBy")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first().and_then(|v| v.as_str()).map(|s| s.to_string())),
        has_metadata: json.contains("[METADATA]"),
    })
}

pub struct Aria2Status {
    pub status: String,
    pub completed_length: u64,
    pub total_length: u64,
    pub info_hash: Option<String>,
    pub followed_by: Option<String>,
    pub has_metadata: bool,
}

pub fn find_file_download_by_info_hash(rpc_url: &str, info_hash: &str) -> Option<String> {
    for method in &["aria2.tellActive", "aria2.tellWaiting"] {
        let params = if *method == "aria2.tellWaiting" {
            "[0, 100]".to_string()
        } else {
            "[]".to_string()
        };

        if let Some(json) = rpc_call(rpc_url, method, &params) {
            if let Ok(val) = serde_json::from_str::<Value>(&json) {
                if let Some(results) = val.get("result").and_then(|v| v.as_array()) {
                    for entry in results {
                        if let Some(entry_hash) = entry.get("infoHash").and_then(|v| v.as_str()) {
                            if entry_hash == info_hash && !json.contains("[METADATA]") {
                                if let Some(gid) = entry.get("gid").and_then(|v| v.as_str()) {
                                    return Some(gid.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
