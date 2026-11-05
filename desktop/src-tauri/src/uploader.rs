use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use reqwest::{
    blocking::{
        Client, RequestBuilder,
        multipart::{Form, Part},
    },
    header::CONTENT_TYPE,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

pub const MARKER: &str = "Sending decks to server: ";
pub const SERVER: &str = "api.crocodyle.fr";
pub const STABLE: Duration = Duration::from_secs(5);
pub const RETRY: Duration = Duration::from_secs(600);
pub const CLOSE_TIMEOUT: Duration = Duration::from_secs(2);
pub const MAX_CHUNK: usize = 900_000;

pub fn default_folder(windows: bool, var: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    let var = |key: &str| var(key).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = if windows {
        var("USERPROFILE")?.join("AppData").join("LocalLow")
    } else {
        var("XDG_CONFIG_HOME")
            .filter(|p| p.is_absolute())
            .or_else(|| Some(var("HOME")?.join(".config")))?
            .join("unity3d")
    };
    Some(base.join("Ticroco").join("Spam"))
}

pub fn account_dirs(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(folder.join("replays").join(SERVER)) else { return Vec::new() };
    let mut dirs: Vec<PathBuf> =
        entries.flatten().map(|e| e.path()).filter(|p| p.join("index.json").is_file()).collect();
    dirs.sort();
    dirs
}

pub fn replay_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let is_replay =
        |name: &str| name.len() > "match-.srpl".len() && name.starts_with("match-") && name.ends_with(".srpl");
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && path.file_name().and_then(|n| n.to_str()).is_some_and(is_replay) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

pub fn deck_lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes).lines().filter_map(|l| l.find(MARKER).map(|at| l[at..].to_string())).collect()
}

pub fn chunk_len(lines: &[String], max: usize) -> usize {
    let mut size = 0;
    lines
        .iter()
        .position(|line| {
            size += line.len() + 1;
            size > max + 1
        })
        .unwrap_or(lines.len())
        .max(1)
}

#[derive(Default)]
pub struct LogTail {
    offset: u64,
}

impl LogTail {
    pub fn read(&mut self, path: &Path) -> io::Result<Vec<String>> {
        let mut file = match File::open(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            file => file?,
        };
        let len = file.metadata()?.len();
        if len < self.offset {
            self.offset = 0;
        }
        file.seek(SeekFrom::Start(self.offset))?;
        let mut buf = Vec::new();
        file.take(len - self.offset).read_to_end(&mut buf)?;
        let end = buf.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
        self.offset += end as u64;
        Ok(deck_lines(&buf[..end]))
    }
}

#[derive(Default)]
pub struct Watch {
    seen: Option<(SystemTime, u64)>,
    due: Option<Instant>,
}

impl Watch {
    pub fn due(&mut self, meta: Option<(SystemTime, u64)>, now: Instant) -> bool {
        if meta != self.seen {
            self.seen = meta;
            self.due = meta.map(|_| now + STABLE);
        }
        self.due.is_some_and(|due| now >= due)
    }
}

pub fn older(current: &str, min: &str) -> bool {
    let parse = |v: &str| v.trim().split('.').map(|p| p.parse::<u64>().ok()).collect::<Option<Vec<_>>>();
    let (Some(mut a), Some(mut b)) = (parse(current), parse(min)) else { return false };
    let len = a.len().max(b.len());
    a.resize(len, 0);
    b.resize(len, 0);
    a < b
}

#[derive(Debug)]
pub struct Fail {
    pub status: Option<u16>,
    pub message: String,
}

impl Fail {
    pub fn retry(&self) -> bool {
        self.status.is_none_or(|s| s == 429 || s >= 500)
    }
}

impl From<io::Error> for Fail {
    fn from(e: io::Error) -> Self {
        Self { status: None, message: e.to_string() }
    }
}

impl From<reqwest::Error> for Fail {
    fn from(e: reqwest::Error) -> Self {
        Self { status: None, message: e.to_string() }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Deserialize)]
struct Item {
    status: String,
    reason: Option<String>,
}

pub struct Api {
    http: Client,
    site: String,
}

impl Api {
    pub fn new(site: &str) -> Self {
        let http = Client::builder().timeout(Duration::from_secs(60)).build().expect("HTTP client");
        Self { http, site: site.trim_end_matches('/').to_string() }
    }

    fn post(&self, path: &str) -> RequestBuilder {
        self.http.post(format!("{}{path}", self.site))
    }

    fn call<T: DeserializeOwned>(&self, req: RequestBuilder) -> Result<T, Fail> {
        #[derive(Deserialize)]
        struct ErrorBody {
            error: String,
        }
        let res = req.send()?;
        let status = res.status();
        if status.is_success() {
            return Ok(res.json()?);
        }
        let message = res.json::<ErrorBody>().map_or_else(|_| status.to_string(), |b| b.error);
        Err(Fail { status: Some(status.as_u16()), message })
    }

    pub fn device_code(&self) -> Result<DeviceCode, Fail> {
        self.call(self.post("/api/device/code"))
    }

    pub fn device_token(&self, device_code: &str) -> Result<Option<String>, Fail> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Token {
            access_token: String,
        }
        let req = self.post("/api/device/token").json(&serde_json::json!({ "deviceCode": device_code }));
        match self.call::<Token>(req) {
            Ok(t) => Ok(Some(t.access_token)),
            Err(fail) if fail.message == "authorization_pending" => Ok(None),
            Err(fail) => Err(fail),
        }
    }

    pub fn min_version(&self) -> Result<String, Fail> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct MinVersion {
            min_version: String,
        }
        let req = self.http.get(format!("{}/api/desktop/min-version", self.site));
        Ok(self.call::<MinVersion>(req)?.min_version)
    }

    pub fn presence(&self, token: &str, event: &str, version: &str) -> Result<String, Fail> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct MinVersion {
            min_version: String,
        }
        let mut req = self
            .post("/api/desktop/presence")
            .bearer_auth(token)
            .json(&serde_json::json!({ "event": event, "version": version }));
        if event == "close" {
            req = req.timeout(CLOSE_TIMEOUT);
        }
        Ok(self.call::<MinVersion>(req)?.min_version)
    }

    fn upload(&self, token: &str, dir: &Path, index: Vec<u8>) -> Result<Vec<Item>, Fail> {
        #[derive(Deserialize)]
        struct UploadResult {
            entries: Vec<Item>,
        }
        let mut form = Form::new().part("index", Part::bytes(index).file_name("index.json"));
        for path in replay_files(dir)? {
            let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            form = form.part("replay", Part::bytes(fs::read(&path)?).file_name(name));
        }
        let req = self.post("/api/matches").bearer_auth(token).multipart(form);
        Ok(self.call::<UploadResult>(req)?.entries)
    }

    fn import_decks(&self, token: &str, lines: &[String]) -> Result<Vec<Item>, Fail> {
        #[derive(Deserialize)]
        struct ImportResult {
            decks: Vec<Item>,
        }
        let req = self
            .post("/api/decks/import")
            .bearer_auth(token)
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(lines.join("\n"));
        Ok(self.call::<ImportResult>(req)?.decks)
    }
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Matches,
    Decks,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub kind: Kind,
    pub at: u64,
    pub counts: BTreeMap<String, usize>,
    pub reasons: Vec<String>,
    pub error: Option<String>,
    #[serde(skip)]
    pub status: Option<u16>,
}

impl Report {
    pub fn unauthorized(&self) -> bool {
        self.status == Some(401)
    }

    fn new(kind: Kind, result: Result<Vec<Item>, Fail>) -> Self {
        let at = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64);
        let mut report = Self { kind, at, counts: BTreeMap::new(), reasons: Vec::new(), error: None, status: None };
        match result {
            Ok(items) => {
                for item in items {
                    if let Some(reason) = item.reason.filter(|_| item.status == "rejected")
                        && !report.reasons.contains(&reason)
                    {
                        report.reasons.push(reason);
                    }
                    *report.counts.entry(item.status).or_default() += 1;
                }
            }
            Err(fail) => {
                report.error = Some(fail.message);
                report.status = fail.status;
            }
        }
        report
    }
}

#[derive(Default)]
struct Account {
    watch: Watch,
    sent: Option<Vec<u8>>,
}

pub struct Uploader {
    folder: PathBuf,
    accounts: BTreeMap<PathBuf, Account>,
    started: bool,
    log: LogTail,
    decks: Vec<String>,
    decks_due: Option<Instant>,
}

impl Uploader {
    pub fn new(folder: PathBuf) -> Self {
        Self {
            folder,
            accounts: BTreeMap::new(),
            started: false,
            log: LogTail::default(),
            decks: Vec::new(),
            decks_due: None,
        }
    }

    pub fn step(&mut self, api: &Api, token: &str, now: Instant) -> Vec<Report> {
        let mut reports = Vec::new();
        for dir in account_dirs(&self.folder) {
            let index = dir.join("index.json");
            let meta = fs::metadata(&index).ok().map(|m| (m.modified().unwrap_or(UNIX_EPOCH), m.len()));
            let account = self.accounts.entry(dir.clone()).or_default();
            if !account.watch.due(meta, now) {
                continue;
            }
            account.watch.due = None;
            let result = fs::read(&index).map_err(Fail::from).and_then(|bytes| {
                let hash = Sha256::digest(&bytes).to_vec();
                if account.sent.as_ref() == Some(&hash) {
                    return Ok(None);
                }
                api.upload(token, &dir, bytes).map(|items| Some((items, hash)))
            });
            match result {
                Ok(None) => {}
                Ok(Some((items, hash))) => {
                    account.sent = Some(hash);
                    reports.push(Report::new(Kind::Matches, Ok(items)));
                }
                Err(fail) => {
                    if fail.retry() {
                        account.watch.due = Some(now + RETRY);
                    }
                    reports.push(Report::new(Kind::Matches, Err(fail)));
                    if reports.last().is_some_and(Report::unauthorized) {
                        return reports;
                    }
                }
            }
        }

        let mut lines = Vec::new();
        if !self.started {
            self.started = true;
            lines = LogTail::default().read(&self.folder.join("Player-prev.log")).unwrap_or_default();
        }
        lines.extend(self.log.read(&self.folder.join("Player.log")).unwrap_or_default());
        if !lines.is_empty() {
            self.decks.extend(lines);
            self.decks_due = Some(now);
        }
        while !self.decks.is_empty() && self.decks_due.is_some_and(|due| now >= due) {
            let len = chunk_len(&self.decks, MAX_CHUNK);
            let result = api.import_decks(token, &self.decks[..len]);
            match &result {
                Err(fail) if fail.retry() => self.decks_due = Some(now + RETRY),
                _ => drop(self.decks.drain(..len)),
            }
            reports.push(Report::new(Kind::Decks, result));
            if reports.last().is_some_and(Report::unauthorized) {
                break;
            }
        }
        reports
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    const A: &str = "00000000-0000-4000-8000-00000000000a";
    const B: &str = "00000000-0000-4000-8000-00000000000b";
    const DECKS: &str = r#"{"userId":"x","added":[],"updated":[],"removed":[]}"#;

    fn line() -> String {
        format!("<color=#FFA000>[UserData]</color> {MARKER}{DECKS}")
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("spam-live-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn touch(path: &Path) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"x").unwrap();
    }

    #[test]
    fn default_folder_per_os() {
        let env = |pairs: &'static [(&str, &str)]| {
            move |k: &str| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| OsString::from(v))
        };
        let path = |parts: &[&str]| parts.iter().collect::<PathBuf>();
        assert_eq!(
            default_folder(true, env(&[("USERPROFILE", "C:/Users/me")])),
            Some(path(&["C:/Users/me", "AppData", "LocalLow", "Ticroco", "Spam"]))
        );
        assert_eq!(default_folder(true, env(&[("HOME", "/home/me")])), None);
        assert_eq!(
            default_folder(false, env(&[("HOME", "/home/me")])),
            Some(path(&["/home/me", ".config", "unity3d", "Ticroco", "Spam"]))
        );
        assert_eq!(
            default_folder(false, env(&[("HOME", "/home/me"), ("XDG_CONFIG_HOME", "/cfg")])),
            Some(path(&["/cfg", "unity3d", "Ticroco", "Spam"]))
        );
        assert_eq!(
            default_folder(false, env(&[("HOME", "/home/me"), ("XDG_CONFIG_HOME", "rel")])),
            Some(path(&["/home/me", ".config", "unity3d", "Ticroco", "Spam"]))
        );
        assert_eq!(default_folder(false, env(&[("HOME", "")])), None);
    }

    #[test]
    fn selects_only_index_and_replays() {
        let root = temp("select");
        let server = root.join("replays").join(SERVER);
        for f in [
            format!("{B}/index.json"),
            format!("{B}/match-2.srpl"),
            format!("{A}/index.json"),
            format!("{A}/match-1.srpl"),
            format!("{A}/match-.srpl"),
            format!("{A}/other.srpl"),
            format!("{A}/match-3.srpl.tmp"),
            format!("{A}/old/match-0.srpl"),
            "no-index/match-4.srpl".into(),
        ] {
            touch(&server.join(f));
        }
        fs::create_dir_all(server.join(A).join("match-5.srpl")).unwrap();
        touch(&root.join("replays/test.crocodyle.fr").join(A).join("index.json"));
        touch(&root.join("usage_counters.json"));

        assert_eq!(account_dirs(&root), vec![server.join(A), server.join(B)]);
        assert_eq!(replay_files(&server.join(A)).unwrap(), vec![server.join(A).join("match-1.srpl")]);
        assert_eq!(replay_files(&server.join(B)).unwrap(), vec![server.join(B).join("match-2.srpl")]);
        assert!(account_dirs(&root.join("missing")).is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keeps_deck_lines_from_the_marker() {
        let text = format!("Initialize engine version\r\n{}\r\nGfxDevice: creating device client\r\n", line());
        assert_eq!(deck_lines(text.as_bytes()), vec![format!("{MARKER}{DECKS}")]);
        assert!(deck_lines(b"Sending decks to server:{}\n").is_empty());
    }

    #[test]
    fn log_tail_reads_complete_new_lines() {
        let dir = temp("tail");
        let path = dir.join("Player.log");
        let mut tail = LogTail::default();
        assert!(tail.read(&path).unwrap().is_empty());

        fs::write(&path, format!("start\n{}1\n{}2", line(), line())).unwrap();
        assert_eq!(tail.read(&path).unwrap(), vec![format!("{MARKER}{DECKS}1")]);
        assert!(tail.read(&path).unwrap().is_empty());

        let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
        write!(file, "\r\nother\n{}3\n", line()).unwrap();
        assert_eq!(tail.read(&path).unwrap(), vec![format!("{MARKER}{DECKS}2"), format!("{MARKER}{DECKS}3")]);

        fs::write(&path, format!("{}4\n", line())).unwrap();
        assert_eq!(tail.read(&path).unwrap(), vec![format!("{MARKER}{DECKS}4")]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn splits_deck_lines_in_chunks() {
        let lines = ["aaaa", "bbbb", "cc", "dddddddddddd", "e"].map(String::from);
        let sizes = |max| {
            let mut rest = &lines[..];
            let mut sizes = Vec::new();
            while !rest.is_empty() {
                let len = chunk_len(rest, max);
                assert!(len == 1 || rest[..len].join("\n").len() <= max);
                sizes.push(len);
                rest = &rest[len..];
            }
            sizes
        };
        assert_eq!(sizes(9), [2, 1, 1, 1]);
        assert_eq!(sizes(8), [1, 2, 1, 1]);
        assert_eq!(sizes(26), [4, 1]);
        assert_eq!(sizes(28), [5]);
    }

    #[test]
    fn keeps_deck_lines_for_a_retry() {
        let folder = temp("retry");
        fs::write(folder.join("Player.log"), format!("{}1\n{}2\n", line(), line())).unwrap();
        let api = Api::new("http://127.0.0.1:9");
        let mut uploader = Uploader::new(folder.clone());
        let t0 = Instant::now();
        let reports = uploader.step(&api, "token", t0);
        assert_eq!(reports.len(), 1);
        assert!(reports[0].error.is_some() && reports[0].kind == Kind::Decks);
        assert_eq!(uploader.decks.len(), 2);
        assert!(uploader.step(&api, "token", t0 + STABLE).is_empty());
        assert_eq!(uploader.step(&api, "token", t0 + RETRY).len(), 1);
        assert_eq!(uploader.decks.len(), 2);
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn compares_versions() {
        assert!(older("0.1.0", "0.2.0"));
        assert!(older("1.9.0", "1.10.0"));
        assert!(older("1.0", "1.0.1"));
        assert!(!older("1.0.0", "1.0"));
        assert!(!older("0.2.0", "0.1.9"));
        assert!(!older("0.1.0", "0.0.0"));
        assert!(!older("0.1.0", "next"));
    }

    #[test]
    fn waits_for_a_stable_file() {
        let t0 = Instant::now();
        let s = |secs| t0 + Duration::from_secs(secs);
        let a = Some((UNIX_EPOCH, 10));
        let b = Some((UNIX_EPOCH + Duration::from_secs(1), 12));
        let mut watch = Watch::default();
        assert!(!watch.due(None, s(0)));
        assert!(!watch.due(a, s(0)));
        assert!(!watch.due(a, s(4)));
        assert!(!watch.due(b, s(4)));
        assert!(!watch.due(b, s(8)));
        assert!(watch.due(b, s(9)));
        watch.due = None;
        assert!(!watch.due(b, s(20)));
        assert!(!watch.due(None, s(21)));
        assert!(!watch.due(b, s(25)));
        assert!(!watch.due(b, s(29)));
        assert!(watch.due(b, s(30)));
    }

    #[test]
    fn retries_only_transient_failures() {
        let fail = |status| Fail { status, message: String::new() };
        assert!(fail(None).retry());
        assert!(fail(Some(429)).retry());
        assert!(fail(Some(502)).retry());
        assert!(!fail(Some(400)).retry());
        assert!(!fail(Some(401)).retry());
        assert!(!fail(Some(403)).retry());
    }

    fn copy(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap().flatten() {
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    #[test]
    #[ignore = "needs a running API, SPAM_SITE_URL, SPAM_TEST_SESSION and samples/save/"]
    fn pairs_and_uploads_samples() {
        let site = std::env::var("SPAM_SITE_URL").expect("SPAM_SITE_URL");
        let session = std::env::var("SPAM_TEST_SESSION").expect("SPAM_TEST_SESSION");
        let api = Api::new(&site);

        let code = api.device_code().unwrap();
        assert_eq!(api.device_token(&code.device_code).unwrap(), None);
        let confirm = Client::new()
            .post(format!("{site}/api/device/confirm"))
            .header("Cookie", format!("session={session}"))
            .header("Origin", &site)
            .json(&serde_json::json!({ "userCode": code.user_code }))
            .send()
            .unwrap();
        assert_eq!(confirm.status().as_u16(), 204);
        let token = api.device_token(&code.device_code).unwrap().expect("token");
        assert!(api.presence(&token, "open", "0.1.0").is_ok());

        let folder = temp("samples");
        copy(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples/save"), &folder);
        let mut uploader = Uploader::new(folder.clone());
        let t0 = Instant::now();
        let mut reports = uploader.step(&api, &token, t0);
        reports.extend(uploader.step(&api, &token, t0 + STABLE));

        let index = account_dirs(&folder)[0].join("index.json");
        File::options().write(true).open(&index).unwrap().set_modified(SystemTime::now() + RETRY).unwrap();
        assert!(uploader.step(&api, &token, t0 + STABLE * 2).is_empty());
        assert!(uploader.step(&api, &token, t0 + STABLE * 3).is_empty());
        fs::remove_dir_all(folder).unwrap();

        let counts = |kind| -> BTreeMap<String, usize> {
            let report = reports.iter().find(|r| r.kind == kind).expect("report");
            assert_eq!(report.error, None);
            report.counts.clone()
        };
        let matches = counts(Kind::Matches);
        assert!(matches.keys().all(|k| k == "new" || k == "known"), "{matches:?}");
        assert!(matches.values().sum::<usize>() > 0);
        assert!(counts(Kind::Decks).values().sum::<usize>() > 0);
        assert!(api.presence(&token, "close", "0.1.0").is_ok());
        assert_eq!(api.presence("bad", "check", "0.1.0").unwrap_err().status, Some(401));
    }
}
