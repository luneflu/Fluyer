//! Solves the player's `n` (throttle) and `sig` (signature) transforms.
//!
//! The transform only exists as obfuscated JS in the player script, so a JS engine is
//! the honest answer: QuickJS (`rquickjs`) runs yt-dlp's EJS solver (`assets/ejs/`,
//! Unlicense) over a *preprocessed* copy of the player, cached on disk per player
//! version. No V8, no Node, no yt-dlp process.
//!
//! One worker thread owns the QuickJS runtime (it is not `Send`) and exits after
//! [`IDLE`] so its ~20 MB goes back to the OS.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use rquickjs::{Context, Runtime};

/// yt-dlp's EJS solver, vendored (Unlicense). See `assets/ejs/README.md`.
const JSC: &str = include_str!("../../../assets/ejs/jsc.min.js");
/// A track's solve and the next one's prefetch land within seconds of each other.
const IDLE: Duration = Duration::from_secs(20);
/// YouTube rolls a new player out every few days; one hour is close enough.
const PLAYER_TTL: Duration = Duration::from_secs(3600);
/// A hung player script must not block every later solve.
const JS_LIMIT: Duration = Duration::from_secs(60);
const MAX_STACK: usize = 64 << 20;

#[derive(Clone, Debug, PartialEq)]
pub struct Player {
    pub id: String,
    pub signature_timestamp: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Solved {
    pub n: Option<String>,
    pub sig: Option<String>,
}

struct Job {
    path: PathBuf,
    n: Option<String>,
    sig: Option<String>,
    reply: Sender<Result<Solved, String>>,
}

pub struct Solver {
    dir: PathBuf,
    worker: Mutex<Option<Sender<Job>>>,
    player: Mutex<Option<(Player, Instant)>>,
    /// Keeps a repeated failure from re-downloading the player for every track.
    failed_at: Mutex<Option<Instant>>,
}

impl Solver {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            dir: data_dir.join("ejs"),
            worker: Mutex::new(None),
            player: Mutex::new(None),
            failed_at: Mutex::new(None),
        }
    }

    /// Current player version, preprocessed and ready to solve with.
    pub async fn player(&self, http: &reqwest::Client) -> Result<Player, String> {
        if let Some((player, at)) = self.player.lock().expect("solver").as_ref() {
            if at.elapsed() < PLAYER_TTL {
                return Ok(player.clone());
            }
        }
        if self
            .failed_at
            .lock()
            .expect("solver")
            .is_some_and(|at| at.elapsed() < Duration::from_secs(600))
        {
            return Err("player failed to prepare a moment ago".to_string());
        }
        match self.prepare(http).await {
            Ok(player) => {
                *self.player.lock().expect("solver") = Some((player.clone(), Instant::now()));
                Ok(player)
            }
            Err(error) => {
                *self.failed_at.lock().expect("solver") = Some(Instant::now());
                Err(error)
            }
        }
    }

    async fn prepare(&self, http: &reqwest::Client) -> Result<Player, String> {
        let iframe = get_text(http, "https://www.youtube.com/iframe_api").await?;
        let id = player_id(&iframe).ok_or("no player version in iframe_api")?;
        let script = self.dir.join(format!("{id}.js"));
        let sts_file = self.dir.join(format!("{id}.sts"));
        if let (Ok(sts), true) = (
            std::fs::read_to_string(&sts_file).map_err(|e| e.to_string()),
            script.exists(),
        ) {
            if let Ok(sts) = sts.trim().parse() {
                return Ok(Player { id, signature_timestamp: sts });
            }
        }
        let url = format!("https://www.youtube.com/s/player/{id}/player_ias.vflset/en_US/base.js");
        let base = get_text(http, &url).await?;
        let signature_timestamp = signature_timestamp(&base).ok_or("no signatureTimestamp in player")?;
        let preprocessed = tokio::task::spawn_blocking(move || preprocess(&base))
            .await
            .map_err(|e| e.to_string())??;
        // Keep only the current player: each preprocessed copy is a few MB.
        if let Ok(entries) = std::fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                let _ = std::fs::remove_file(entry.path());
            }
        }
        std::fs::create_dir_all(&self.dir).map_err(|e| format!("ejs dir: {e}"))?;
        std::fs::write(&script, preprocessed).map_err(|e| format!("cached player: {e}"))?;
        std::fs::write(&sts_file, signature_timestamp.to_string()).map_err(|e| format!("cached sts: {e}"))?;
        Ok(Player { id, signature_timestamp })
    }

    /// Rewrites the `n`/`sig` parts of a player response's formats in place.
    pub async fn solve_streaming(&self, http: &reqwest::Client, body: &mut serde_json::Value) -> Result<(), String> {
        let jobs = collect_jobs(body);
        if jobs.is_empty() {
            return Ok(());
        }
        let player = self.player(http).await?;
        for (n, sig) in jobs {
            let solved = self.solve(&player, n.clone(), sig.clone()).await?;
            apply_solution(body, n, sig, &solved);
        }
        Ok(())
    }

    pub async fn solve(&self, player: &Player, n: Option<String>, sig: Option<String>) -> Result<Solved, String> {
        let path = self.dir.join(format!("{}.js", player.id));
        let (reply, rx) = mpsc::channel();
        self.send(Job { path, n, sig, reply })?;
        rx.recv().map_err(|_| "solver dropped the job".to_string())?
    }

    /// Hands a job to the worker thread, starting it again if it went idle.
    fn send(&self, job: Job) -> Result<(), String> {
        let mut worker = self.worker.lock().expect("solver worker");
        let mut pending = Some(job);
        if let Some(tx) = worker.as_ref() {
            match tx.send(pending.take().expect("job present")) {
                Ok(()) => return Ok(()),
                // The worker exited on its idle timeout; take the job back and restart it.
                Err(error) => pending = Some(error.0),
            }
        }
        let (tx, rx) = mpsc::channel::<Job>();
        std::thread::Builder::new()
            .name("ytmusic-jsc".into())
            .spawn(move || run_worker(rx))
            .map_err(|e| format!("could not start solver thread: {e}"))?;
        tx.send(pending.take().expect("job present"))
            .map_err(|_| "solver thread failed to start".to_string())?;
        *worker = Some(tx);
        Ok(())
    }
}

/// Owns the QuickJS runtime; exits after `IDLE` without work.
fn run_worker(rx: mpsc::Receiver<Job>) {
    let mut loaded: Option<(PathBuf, Runtime, Context)> = None;
    loop {
        match rx.recv_timeout(IDLE) {
            Ok(job) => {
                if loaded.as_ref().is_none_or(|(path, ..)| *path != job.path) {
                    loaded = None;
                    match load(&job.path) {
                        Ok((runtime, context)) => loaded = Some((job.path.clone(), runtime, context)),
                        Err(error) => {
                            let _ = job.reply.send(Err(error));
                            continue;
                        }
                    }
                }
                let Some((_, runtime, context)) = loaded.as_ref() else { continue };
                watchdog(runtime);
                let _ = job.reply.send(solve(context, job.n, job.sig));
            }
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => return,
        }
    }
}

fn load(path: &Path) -> Result<(Runtime, Context), String> {
    let preprocessed = std::fs::read_to_string(path).map_err(|e| format!("player not cached: {e}"))?;
    let runtime = Runtime::new().map_err(|e| e.to_string())?;
    runtime.set_max_stack_size(MAX_STACK);
    watchdog(&runtime);
    let context = Context::full(&runtime).map_err(|e| e.to_string())?;
    // The preprocessed player is a function body that fills `_result`; evaluating it
    // once gives direct callables, so a solve is one call instead of re-parsing 4 MB.
    let source = format!("var __r = {{n: null, sig: null}};\n(function (_result) {{\n{preprocessed}\n}})(__r);");
    context.with(|ctx| ctx.eval::<(), _>(source).map_err(|e| js_error(&ctx, e)))?;
    Ok((runtime, context))
}

fn solve(context: &Context, n: Option<String>, sig: Option<String>) -> Result<Solved, String> {
    context.with(|ctx| {
        let run = |method: &str, input: Option<String>| -> Result<Option<String>, String> {
            let Some(input) = input else { return Ok(None) };
            let code = format!("__r.{method}({})", serde_json::to_string(&input).map_err(|e| e.to_string())?);
            ctx.eval::<String, _>(code).map(Some).map_err(|e| js_error(&ctx, e))
        };
        Ok(Solved { n: run("n", n)?, sig: run("sig", sig)? })
    })
}

fn js_error(ctx: &rquickjs::Ctx, error: rquickjs::Error) -> String {
    if matches!(error, rquickjs::Error::Exception) {
        let caught = ctx.catch();
        let text = caught
            .as_exception()
            .map(|e| e.to_string())
            .or_else(|| caught.as_string().and_then(|s| s.to_string().ok()))
            .unwrap_or_else(|| "exception".into());
        return format!("js: {text}");
    }
    format!("js: {error}")
}

fn watchdog(runtime: &Runtime) {
    let deadline = Instant::now() + JS_LIMIT;
    runtime.set_interrupt_handler(Some(Box::new(move || Instant::now() >= deadline)));
}

/// Parses the player and extracts its solver functions into a standalone script.
fn preprocess(base: &str) -> Result<String, String> {
    let runtime = Runtime::new().map_err(|e| e.to_string())?;
    runtime.set_max_stack_size(MAX_STACK);
    watchdog(&runtime);
    let context = Context::full(&runtime).map_err(|e| e.to_string())?;
    context.with(|ctx| {
        let input = serde_json::json!({
            "type": "player",
            "player": base,
            "requests": [],
            "output_preprocessed": true,
        });
        let source = format!("{JSC}\nJSON.stringify(jsc.default({input}))");
        let output: String = ctx.eval(source).map_err(|e| js_error(&ctx, e))?;
        let value: serde_json::Value = serde_json::from_str(&output).map_err(|e| e.to_string())?;
        value["preprocessed_player"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| format!("solver returned no player: {}", &output[..output.len().min(200)]))
    })
}

/// `n` and `sig` pairs still to solve, taken from `adaptiveFormats`/`formats`.
fn collect_jobs(body: &serde_json::Value) -> Vec<(Option<String>, Option<String>)> {
    let mut jobs = Vec::new();
    for section in ["adaptiveFormats", "formats"] {
        let Some(items) = body
            .get("streamingData")
            .and_then(|data| data.get(section))
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };
        for item in items {
            let n = item
                .get("url")
                .and_then(serde_json::Value::as_str)
                .and_then(url_param)
                .filter(|n| needs_solving(n));
            let sig = item
                .get("signatureCipher")
                .or_else(|| item.get("cipher"))
                .and_then(serde_json::Value::as_str)
                .and_then(signature_from_cipher);
            if n.is_some() || sig.is_some() {
                jobs.push((n, sig));
            }
        }
    }
    jobs
}

/// Replaces the solved values in the response, so callers get playable URLs.
fn apply_solution(
    body: &mut serde_json::Value,
    n: Option<String>,
    sig: Option<String>,
    solved: &Solved,
) {
    let Some(data) = body.get_mut("streamingData") else { return };
    for section in ["adaptiveFormats", "formats"] {
        let Some(items) = data.get_mut(section).and_then(serde_json::Value::as_array_mut) else {
            continue;
        };
        for item in items.iter_mut() {
            if let (Some(old), Some(new)) = (n.as_deref(), solved.n.as_deref()) {
                if let Some(url) = item.get("url").and_then(serde_json::Value::as_str) {
                    if url_param(url) == Some(old.to_string()) {
                        if let Some(replaced) = replace_param(url, "n", new) {
                            item["url"] = serde_json::Value::String(replaced);
                        }
                    }
                }
            }
            if let (Some(old_sig), Some(new_sig)) = (sig.as_deref(), solved.sig.as_deref()) {
                if let Some(cipher) = item.get("signatureCipher").and_then(serde_json::Value::as_str) {
                    if signature_from_cipher(cipher).as_deref() == Some(old_sig) {
                        if let Some(replaced) = replace_cipher(cipher, new_sig) {
                            item["signatureCipher"] = serde_json::Value::String(replaced);
                        }
                    }
                }
            }
        }
    }
}

/// `n` value out of a googlevideo URL.
pub fn url_param(url: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix("n="))
        .map(|value| value.split('&').next().unwrap_or(value).to_string())
}

fn replace_param(url: &str, name: &str, value: &str) -> Option<String> {
    let (head, query) = url.split_once('?')?;
    let mut parts: Vec<String> = Vec::new();
    let mut replaced = false;
    for pair in query.split('&') {
        if pair.starts_with(&format!("{name}=")) {
            parts.push(format!("{name}={value}"));
            replaced = true;
        } else {
            parts.push(pair.to_string());
        }
    }
    replaced.then(|| format!("{head}?{}", parts.join("&")))
}

/// `s=…&sp=sig&url=…` → the `s` value.
pub fn signature_from_cipher(cipher: &str) -> Option<String> {
    let decoded = urlencoding_decode(cipher);
    decoded
        .split('&')
        .find_map(|pair| pair.strip_prefix("s="))
        .map(str::to_string)
}

fn replace_cipher(cipher: &str, signature: &str) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut replaced = false;
    for pair in cipher.split('&') {
        if pair.starts_with("s=") {
            parts.push(format!("s={}", urlencoding_encode(signature)));
            replaced = true;
        } else {
            parts.push(pair.to_string());
        }
    }
    replaced.then(|| parts.join("&"))
}

fn urlencoding_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        index += 3;
                    }
                    Err(_) => {
                        out.push(bytes[index]);
                        index += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn urlencoding_encode(text: &str) -> String {
    text.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// Throttle values are long and opaque; short ones are already fine.
fn needs_solving(value: &str) -> bool {
    value.len() > 8
}

async fn get_text(http: &reqwest::Client, url: &str) -> Result<String, String> {
    let response = http
        .get(url)
        .send()
        .await
        .map_err(|e| format!("{url} failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("{url} returned {}", response.status()));
    }
    response.text().await.map_err(|e| format!("{url} body failed: {e}"))
}

/// `player\/5203c085\/` inside `iframe_api`.
pub fn player_id(iframe: &str) -> Option<String> {
    iframe.match_indices("player").find_map(|(index, _)| {
        let rest = iframe[index + 6..].trim_start_matches(['\\', '/']);
        let id = rest.get(..8)?;
        (id.bytes().all(|b| b.is_ascii_hexdigit()) && rest[8..].starts_with(['\\', '/'])).then(|| id.to_string())
    })
}

/// `signatureTimestamp:20732` (or `sts:20732`) in the player script.
pub fn signature_timestamp(base: &str) -> Option<u32> {
    ["signatureTimestamp", "sts"].iter().find_map(|key| {
        base.match_indices(key).find_map(|(index, _)| {
            let rest = base[index + key.len()..].trim_start().strip_prefix(':')?.trim_start();
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            (digits.len() == 5).then(|| digits.parse().ok()).flatten()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_id_and_signature_timestamp() {
        let iframe = r"var a='https:\/\/www.youtube.com\/s\/player\/5203c085\/www-widgetapi.vflset'";
        assert_eq!(player_id(iframe).as_deref(), Some("5203c085"));
        assert_eq!(signature_timestamp("x={a:1,signatureTimestamp:20732,b:2}"), Some(20732));
        assert_eq!(signature_timestamp("sts:123,sts:20733"), Some(20733));
    }

    #[test]
    fn urls_and_ciphers_are_read_and_rewritten() {
        let url = "https://rr.googlevideo.com/videoplayback?expire=1&n=AbC123xyz&mime=audio%2Fmp4";
        assert_eq!(url_param(url).as_deref(), Some("AbC123xyz"));
        let rewritten = replace_param(url, "n", "solved").unwrap();
        assert!(rewritten.contains("n=solved"));
        assert!(rewritten.contains("expire=1"));
        assert!(rewritten.contains("mime=audio%2Fmp4"));

        let cipher = "s=AA%3D%3D&sp=sig&url=https%3A%2F%2Frr.googlevideo.com%2Fx";
        assert_eq!(signature_from_cipher(cipher).as_deref(), Some("AA=="));
        let replaced = replace_cipher(cipher, "A/B+C=").unwrap();
        assert!(replaced.starts_with("s=A%2FB%2BC%3D&sp=sig&url="));
    }

    #[test]
    fn collecting_and_applying_jobs_only_touches_matching_entries() {
        let mut body = serde_json::json!({"streamingData": {"adaptiveFormats": [
            {"url": "https://x/videoplayback?n=AbCdEfGhIj&x=1"},
            {"url": "https://x/videoplayback?n=short&x=1"},
            {"signatureCipher": "s=OLD&sp=sig&url=https%3A%2F%2Fx"}
        ]}});
        let jobs = collect_jobs(&body);
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].0.as_deref(), Some("AbCdEfGhIj"));
        assert_eq!(jobs[1].1.as_deref(), Some("OLD"));

        apply_solution(&mut body, jobs[0].0.clone(), None, &Solved { n: Some("NEW".into()), sig: None });
        let formats = body["streamingData"]["adaptiveFormats"].as_array().unwrap();
        assert_eq!(formats[0]["url"], "https://x/videoplayback?n=NEW&x=1");
        assert_eq!(formats[1]["url"], "https://x/videoplayback?n=short&x=1");
    }

    #[test]
    fn solver_thread_reports_a_missing_player_instead_of_hanging() {
        let dir = std::env::temp_dir().join(format!("fluyer-jsc-{}", std::process::id()));
        let solver = Solver::new(&dir);
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let http = reqwest::Client::new();
        // No cached player and no network use: it must fail, not block.
        let player = Player { id: "nonexistent".into(), signature_timestamp: 0 };
        let result = runtime.block_on(solver.solve(&player, Some("x".into()), None));
        assert!(result.is_err());
    }
}
