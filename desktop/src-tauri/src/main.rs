#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod uploader;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Mutex, MutexGuard, PoisonError,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use reqwest::{
    Method, Url,
    header::{CONTENT_TYPE, USER_AGENT},
};
use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Emitter, Manager, WindowEvent, Wry,
    ipc::{InvokeBody, Request},
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    utils::config::{Csp, CspDirectiveSources},
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_updater::{Update, UpdaterExt};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

use uploader::{Api, RETRY, Report, Uploader};

const SITE: &str = match option_env!("SPAM_SITE_URL") {
    Some(url) => url,
    None => "http://localhost:5173",
};
const RELEASES: &str = "https://github.com/elwqnn/spam-live/releases/latest";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const KEYRING_SERVICE: &str = "io.github.elwqnn.spamlive";
const POLL: Duration = Duration::from_secs(2);
const API_TIMEOUT: Duration = Duration::from_secs(60);
const DEV_ORIGIN: &str = "http://localhost:1420";
const UPDATE_CHECK: Duration = Duration::from_secs(6 * 60 * 60);
const WINDOW_STATE: StateFlags = StateFlags::SIZE.union(StateFlags::POSITION).union(StateFlags::MAXIMIZED);
const APPIMAGE_VARS: [&str; 11] = [
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "GDK_PIXBUF_MODULE_FILE",
    "GTK_PATH",
    "GTK_EXE_PREFIX",
    "GTK_IM_MODULE_FILE",
    "GIO_EXTRA_MODULES",
    "GSETTINGS_SCHEMA_DIR",
    "GST_PLUGIN_SYSTEM_PATH",
    "PERL5LIB",
    "PYTHONHOME",
];

#[derive(Default)]
struct Shared {
    data: Mutex<Data>,
    pairing: AtomicU64,
    http: reqwest::Client,
}

#[derive(Default)]
struct Data {
    token: Option<String>,
    code: Option<String>,
    verify: Option<String>,
    notice: Option<Notice>,
    folder: Option<PathBuf>,
    custom: bool,
    last: Option<Report>,
    update: bool,
    ready: Option<Update>,
    manual: bool,
    reset: bool,
}

struct Tray {
    menu: Menu<Wry>,
    open: MenuItem<Wry>,
    quit: MenuItem<Wry>,
    update: MenuItem<Wry>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
enum Notice {
    Unlinked,
    Expired,
    Revoked,
    Error(String),
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Status {
    version: &'static str,
    linked: bool,
    code: Option<String>,
    notice: Option<Notice>,
    folder: Option<PathBuf>,
    custom: bool,
    found: bool,
    last: Option<Report>,
    update: bool,
    ready: Option<String>,
    manual: bool,
    autostart: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Answer {
    status: u16,
    content_type: Option<String>,
    body: String,
}

#[derive(Serialize, Deserialize, Default)]
struct Settings {
    folder: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Link {
    Site,
    Me,
    Verify,
    Releases,
}

fn site() -> &'static str {
    SITE.trim_end_matches('/')
}

fn origin(url: &Url) -> String {
    url.origin().ascii_serialization()
}

fn app_url(url: &Url) -> bool {
    match (url.scheme(), url.host_str()) {
        ("tauri", Some("localhost")) | ("http" | "https", Some("tauri.localhost")) => true,
        _ => cfg!(debug_assertions) && origin(url) == DEV_ORIGIN,
    }
}

fn api_url(site: &str, path: &str) -> Option<Url> {
    let route = path.split('?').next()?;
    if !route.starts_with("/api/") || route.contains(['%', '\\', '#']) {
        return None;
    }
    let url = Url::parse(&format!("{site}{path}")).ok()?;
    let first = url.path().strip_prefix("/api/")?.split('/').next()?.to_ascii_lowercase();
    let empty = url.path().split('/').skip(1).any(str::is_empty);
    let same = origin(&url) == origin(&Url::parse(site).ok()?);
    (same && !empty && url.fragment().is_none() && first != "admin" && first != "device").then_some(url)
}

fn api_method(method: &str) -> Option<Method> {
    ["GET", "POST", "PUT", "PATCH", "DELETE"]
        .contains(&method)
        .then(|| Method::from_bytes(method.as_bytes()).ok())
        .flatten()
}

fn data(app: &AppHandle) -> MutexGuard<'_, Data> {
    app.state::<Shared>().inner().data.lock().unwrap_or_else(PoisonError::into_inner)
}

fn keyring() -> keyring::Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, "device-token")
}

fn status(app: &AppHandle) -> Status {
    let autostart = app.autolaunch().is_enabled().unwrap_or(false);
    let d = data(app);
    Status {
        version: VERSION,
        linked: d.token.is_some(),
        code: d.code.clone(),
        notice: d.notice.clone(),
        found: d.folder.as_ref().is_some_and(|f| f.is_dir()),
        folder: d.folder.clone(),
        custom: d.custom,
        last: d.last.clone(),
        update: d.update,
        ready: d.ready.as_ref().map(|u| u.version.clone()),
        manual: d.manual,
        autostart,
    }
}

fn emit(app: &AppHandle) {
    let _ = app.emit("status", status(app));
}

fn show(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn appimage_env(appdir: &str, var: impl Fn(&str) -> Option<String>) -> Vec<(&'static str, Option<String>)> {
    let appdir = appdir.trim_end_matches('/');
    if appdir.is_empty() {
        return Vec::new();
    }
    let outside = |entry: &&str| !entry.is_empty() && *entry != appdir && !entry.starts_with(&format!("{appdir}/"));
    let mut env: Vec<_> = APPIMAGE_VARS.iter().map(|&key| (key, None)).collect();
    for key in ["PATH", "XDG_DATA_DIRS"] {
        if let Some(value) = var(key) {
            let kept = value.split(':').filter(outside).collect::<Vec<_>>().join(":");
            env.push((key, Some(kept).filter(|k| !k.is_empty())));
        }
    }
    env
}

fn open_url(url: &str) {
    let mut cmd = if cfg!(windows) {
        let mut cmd = Command::new("rundll32");
        cmd.arg("url.dll,FileProtocolHandler");
        cmd
    } else {
        Command::new("xdg-open")
    };
    if cfg!(target_os = "linux")
        && let Ok(appdir) = std::env::var("APPDIR")
    {
        for (key, value) in appimage_env(&appdir, |key| std::env::var(key).ok()) {
            match value {
                Some(value) => cmd.env(key, value),
                None => cmd.env_remove(key),
            };
        }
    }
    if let Ok(mut child) = cmd.arg(url).spawn() {
        thread::spawn(move || child.wait());
    }
}

fn settings_file(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|dir| dir.join("settings.json"))
}

fn cache_file(app: &AppHandle, name: &str) -> Option<PathBuf> {
    let dir = app.path().app_cache_dir().ok()?;
    fs::create_dir_all(&dir).ok()?;
    Some(dir.join(name))
}

fn apply_folder(app: &AppHandle, custom: Option<PathBuf>) {
    let mut d = data(app);
    d.custom = custom.is_some();
    d.folder = custom.or_else(|| uploader::default_folder(cfg!(windows), |key| std::env::var_os(key)));
    d.reset = true;
}

#[tauri::command(async)]
fn get_status(app: AppHandle) -> Status {
    status(&app)
}

#[tauri::command]
fn pair(app: AppHandle) {
    let id = app.state::<Shared>().pairing.fetch_add(1, Ordering::SeqCst) + 1;
    data(&app).notice = None;
    thread::spawn(move || pairing(&app, id));
}

#[tauri::command]
fn cancel_pair(app: AppHandle) {
    app.state::<Shared>().pairing.fetch_add(1, Ordering::SeqCst);
    {
        let mut d = data(&app);
        d.code = None;
        d.verify = None;
    }
    emit(&app);
}

fn close(token: Option<String>) {
    if let Some(token) = token {
        let _ = thread::spawn(move || Api::new(site()).presence(&token, "close", VERSION)).join();
    }
}

fn revoke(app: &AppHandle, token: &str) {
    let mut d = data(app);
    if d.token.as_deref() == Some(token) {
        let _ = keyring().and_then(|entry| entry.delete_credential());
        d.token = None;
        d.notice = Some(Notice::Revoked);
    }
}

#[tauri::command(async)]
fn unlink(app: AppHandle) -> Result<(), String> {
    let token = data(&app).token.clone();
    close(token);
    match keyring().and_then(|entry| entry.delete_credential()) {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(e) => return Err(e.to_string()),
    }
    {
        let mut d = data(&app);
        d.token = None;
        d.notice = Some(Notice::Unlinked);
    }
    emit(&app);
    Ok(())
}

#[tauri::command(async)]
fn set_folder(app: AppHandle, path: Option<String>) -> Result<(), String> {
    let folder = path.map(|p| PathBuf::from(p.trim())).filter(|p| !p.as_os_str().is_empty());
    let file = settings_file(&app).ok_or("no config folder")?;
    let json = serde_json::to_vec(&Settings { folder: folder.clone() }).map_err(|e| e.to_string())?;
    file.parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::write(&file, json))
        .map_err(|e| e.to_string())?;
    apply_folder(&app, folder);
    emit(&app);
    Ok(())
}

#[tauri::command(async)]
fn set_autostart(app: AppHandle, on: bool) -> Result<(), String> {
    let launcher = app.autolaunch();
    if on { launcher.enable() } else { launcher.disable() }.map_err(|e| e.to_string())?;
    emit(&app);
    Ok(())
}

#[tauri::command]
fn open(app: AppHandle, link: Link) {
    let url = match link {
        Link::Site => site().to_string(),
        Link::Me => format!("{}/me", site()),
        Link::Releases => RELEASES.to_string(),
        Link::Verify => match data(&app).verify.clone() {
            Some(url) => url,
            None => return,
        },
    };
    open_url(&url);
}

#[tauri::command]
async fn api(app: AppHandle, request: Request<'_>) -> Result<Answer, String> {
    let header = |name: &str| request.headers().get(name).and_then(|v| v.to_str().ok());
    let method = header("x-method").and_then(api_method).ok_or("method is not allowed")?;
    let url = header("x-path").and_then(|path| api_url(site(), path)).ok_or("path is not allowed")?;
    let token = data(&app).token.clone();
    let mut req = app
        .state::<Shared>()
        .http
        .request(method, url)
        .timeout(API_TIMEOUT)
        .header(USER_AGENT, format!("SPAM.live/{VERSION} ({})", std::env::consts::OS));
    if let Some(token) = &token {
        req = req.bearer_auth(token);
    }
    if let Some(kind) = header("x-type") {
        req = req.header(CONTENT_TYPE, kind);
    }
    if let InvokeBody::Raw(body) = request.body()
        && !body.is_empty()
    {
        req = req.body(body.clone());
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    let status = res.status().as_u16();
    let content_type = res.headers().get(CONTENT_TYPE).and_then(|v| v.to_str().ok()).map(str::to_owned);
    let body = res.text().await.map_err(|e| e.to_string())?;
    if status == 401
        && let Some(token) = &token
    {
        revoke(&app, token);
        emit(&app);
    }
    Ok(Answer { status, content_type, body })
}

#[tauri::command]
fn browse(url: String) -> Result<(), String> {
    let url = Url::parse(&url).map_err(|e| e.to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("only web links can open".into());
    }
    open_url(url.as_str());
    Ok(())
}

#[tauri::command(async)]
fn restart(app: AppHandle) -> Result<(), String> {
    let update = data(&app).ready.take().ok_or("no update is ready")?;
    let token = data(&app).token.clone();
    close(token);
    apply(&app, update, true)?;
    app.restart()
}

#[tauri::command]
fn tray_labels(app: AppHandle, open: String, quit: String, update: String) {
    if let Some(tray) = app.try_state::<Tray>() {
        let _ = tray.open.set_text(open);
        let _ = tray.quit.set_text(quit);
        let _ = tray.update.set_text(update);
    }
}

fn tray_update(app: &AppHandle, ready: bool) {
    if let Some(tray) = app.try_state::<Tray>() {
        let _ = if ready { tray.menu.prepend(&tray.update) } else { tray.menu.remove(&tray.update) };
    }
}

fn writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".spam-live-{}", std::process::id()));
    let created = fs::OpenOptions::new().write(true).create_new(true).open(&probe).is_ok();
    if created {
        let _ = fs::remove_file(&probe);
    }
    created
}

fn replaceable() -> bool {
    !cfg!(target_os = "linux")
        || std::env::var_os("APPIMAGE").is_some_and(|path| Path::new(&path).parent().is_some_and(writable))
}

async fn download(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint =
        Url::parse(&format!("{}/api/desktop/update/{{{{target}}}}/{{{{arch}}}}/{{{{current_version}}}}", site()))?;
    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint])?
        .configure_client(|client| client.connect_timeout(API_TIMEOUT).read_timeout(API_TIMEOUT))
        .build()?;
    let Some(update) = updater.check().await? else { return Ok(()) };
    if !replaceable() {
        data(app).manual = true;
        emit(app);
        return Ok(());
    }
    let bytes = update.download(|_, _| {}, || {}).await?;
    fs::write(cache_file(app, "update").ok_or("no cache folder")?, bytes)?;
    data(app).ready = Some(update);
    tray_update(app, true);
    emit(app);
    Ok(())
}

fn updates(app: AppHandle) {
    if let Some(file) = cache_file(&app, "update") {
        let _ = fs::remove_file(file);
    }
    loop {
        let idle = {
            let d = data(&app);
            d.ready.is_none() && !d.manual
        };
        if idle {
            let _ = tauri::async_runtime::block_on(download(&app));
        }
        thread::sleep(UPDATE_CHECK);
    }
}

fn install(app: &AppHandle, update: Update, restart: bool) -> Result<(), String> {
    let file = cache_file(app, "update").ok_or("no cache folder")?;
    let bytes = fs::read(&file).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&file);
    let show = cache_file(app, "show").filter(|_| restart);
    if let Some(show) = &show {
        let _ = fs::write(show, "");
    }
    let _ = app.save_window_state(WINDOW_STATE);
    let result = update.restart_after_install(restart).install(bytes).map_err(|e| e.to_string());
    if result.is_err()
        && let Some(show) = &show
    {
        let _ = fs::remove_file(show);
    }
    result
}

fn apply(app: &AppHandle, update: Update, restart: bool) -> Result<(), String> {
    tray_update(app, false);
    let result = install(app, update, restart);
    if result.is_err() {
        data(app).manual = true;
        emit(app);
    }
    result
}

fn pairing(app: &AppHandle, id: u64) {
    let current = || app.state::<Shared>().pairing.load(Ordering::SeqCst) == id;
    let finish = |notice: Option<Notice>| {
        if current() {
            {
                let mut d = data(app);
                d.code = None;
                d.verify = None;
                d.notice = notice;
            }
            emit(app);
        }
    };
    let api = Api::new(site());
    let code = match api.device_code() {
        Ok(code) => code,
        Err(fail) => return finish(Some(Notice::Error(fail.message))),
    };
    if !current() {
        return;
    }
    let verify = if code.verification_url.starts_with(&format!("{}/", site())) {
        Some(code.verification_url)
    } else {
        Url::parse_with_params(&format!("{}/link", site()), [("code", &code.user_code)]).ok().map(String::from)
    };
    if let Some(url) = &verify {
        open_url(url);
    }
    {
        let mut d = data(app);
        d.code = Some(code.user_code);
        d.verify = verify;
    }
    emit(app);
    let deadline = Instant::now() + Duration::from_secs(code.expires_in);
    loop {
        thread::sleep(Duration::from_secs(code.interval.max(1)));
        if !current() {
            return;
        }
        match api.device_token(&code.device_code) {
            Ok(Some(token)) if current() => {
                let notice = match keyring().and_then(|entry| entry.set_password(&token)) {
                    Ok(()) => {
                        let mut d = data(app);
                        d.token = Some(token);
                        d.reset = true;
                        None
                    }
                    Err(e) => Some(Notice::Error(e.to_string())),
                };
                return finish(notice);
            }
            Ok(Some(_)) => return,
            Ok(None) => {}
            Err(fail) if fail.message == "expired_token" || fail.message == "invalid_grant" => {
                return finish(Some(Notice::Expired));
            }
            Err(_) if Instant::now() >= deadline => return finish(Some(Notice::Expired)),
            Err(_) => {}
        }
    }
}

fn worker(app: AppHandle) {
    match keyring().and_then(|entry| entry.get_password()) {
        Ok(token) => data(&app).token = Some(token),
        Err(keyring::Error::NoEntry) => {}
        Err(e) => data(&app).notice = Some(Notice::Error(e.to_string())),
    }
    emit(&app);
    let api = Api::new(site());
    let mut uploader: Option<Uploader> = None;
    let mut version_check = Instant::now();
    let mut checked: Option<String> = None;
    let mut online: Option<String> = None;
    loop {
        let now = Instant::now();
        let token = data(&app).token.clone();
        if now >= version_check || token != checked {
            version_check = now + RETRY;
            checked.clone_from(&token);
            let min = match &token {
                Some(token) => {
                    let event = if online.as_ref() == Some(token) { "check" } else { "open" };
                    let result = api.presence(token, event, VERSION);
                    match &result {
                        Ok(_) => online = Some(token.clone()),
                        Err(fail) if fail.status == Some(401) => {
                            revoke(&app, token);
                            emit(&app);
                        }
                        Err(_) => {}
                    }
                    result
                }
                None => api.min_version(),
            };
            if let Ok(min) = min {
                data(&app).update = uploader::older(VERSION, &min);
                emit(&app);
            }
        }
        let (token, folder) = {
            let mut d = data(&app);
            if std::mem::take(&mut d.reset) {
                uploader = None;
            }
            (d.token.clone(), d.folder.clone())
        };
        if let (Some(token), Some(folder)) = (token, folder) {
            let reports = uploader.get_or_insert_with(|| Uploader::new(folder)).step(&api, &token, now);
            if let Some(last) = reports.last().cloned() {
                if reports.iter().any(Report::unauthorized) {
                    revoke(&app, &token);
                }
                data(&app).last = Some(last);
                emit(&app);
            }
        }
        thread::sleep(POLL);
    }
}

fn csp(context: &mut tauri::Context<Wry>) {
    let security = &mut context.config_mut().app.security;
    let Some(policy) = security.csp.take() else { return };
    let mut map: HashMap<String, CspDirectiveSources> = policy.into();
    if let Ok(url) = Url::parse(site()) {
        for directive in ["connect-src", "img-src"] {
            map.entry(directive.into()).or_default().push(origin(&url));
        }
    }
    security.csp = Some(Csp::DirectiveMap(map));
}

fn main() {
    let mut context = tauri::generate_context!();
    csp(&mut context);
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show(app)))
        .plugin(tauri_plugin_autostart::Builder::new().arg("--minimized").build())
        .plugin(tauri_plugin_window_state::Builder::new().with_state_flags(WINDOW_STATE).build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri::plugin::Builder::<Wry>::new("links")
                .on_navigation(|_, url| {
                    let local = app_url(url);
                    if !local && matches!(url.scheme(), "http" | "https") {
                        open_url(url.as_str());
                    }
                    local
                })
                .build(),
        )
        .manage(Shared::default())
        .invoke_handler(tauri::generate_handler![
            get_status,
            pair,
            cancel_pair,
            unlink,
            set_folder,
            set_autostart,
            open,
            tray_labels,
            api,
            browse,
            restart
        ])
        .setup(|app| {
            let settings: Settings = settings_file(app.handle())
                .and_then(|file| fs::read(file).ok())
                .and_then(|json| serde_json::from_slice(&json).ok())
                .unwrap_or_default();
            apply_folder(app.handle(), settings.folder);
            let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let update = MenuItem::with_id(app, "update", "Update ready, restart", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;
            let mut tray = TrayIconBuilder::new().tooltip("SPAM.live").menu(&menu).on_menu_event(|app, event| {
                match event.id.as_ref() {
                    "open" => show(app),
                    "update" => {
                        let app = app.clone();
                        thread::spawn(move || restart(app));
                    }
                    "quit" => {
                        let token = data(app).token.clone();
                        close(token);
                        let ready = data(app).ready.take();
                        if let Some(update) = ready {
                            let _ = apply(app, update, false);
                        }
                        app.exit(0);
                    }
                    _ => {}
                }
            });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            app.manage(Tray { menu, open, quit, update });
            let restarted = cache_file(app.handle(), "show").is_some_and(|file| fs::remove_file(file).is_ok());
            if restarted || !std::env::args().any(|arg| arg == "--minimized") {
                show(app.handle());
            }
            let handle = app.handle().clone();
            thread::spawn(move || worker(handle));
            if !cfg!(debug_assertions) {
                let handle = app.handle().clone();
                thread::spawn(move || updates(handle));
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(context)
        .expect("failed to run the app");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_the_appimage_env() {
        let var = |key: &str| match key {
            "PATH" => Some("/tmp/.mount_a/usr/bin:/usr/bin:/tmp/.mount_a::/bin".into()),
            "XDG_DATA_DIRS" => Some("/tmp/.mount_a//usr/share:/usr/share:/tmp/.mount_ab/share".into()),
            _ => None,
        };
        let env = appimage_env("/tmp/.mount_a/", var);
        assert_eq!(env.len(), APPIMAGE_VARS.len() + 2);
        assert!(APPIMAGE_VARS.iter().all(|&key| env.contains(&(key, None))));
        assert!(env.contains(&("PATH", Some("/usr/bin:/bin".into()))));
        assert!(env.contains(&("XDG_DATA_DIRS", Some("/usr/share:/tmp/.mount_ab/share".into()))));
        assert!(appimage_env("/", var).is_empty());
    }

    #[test]
    fn api_paths() {
        let site = "http://localhost:5173";
        let ok = |path: &str| api_url(site, path).map(String::from);
        assert_eq!(ok("/api/me").as_deref(), Some("http://localhost:5173/api/me"));
        assert!(ok("/api/me/devices/3").is_some());
        assert!(ok("/api/decks/12/vote").is_some());
        assert!(ok("/api/feedback").is_some());
        assert_eq!(
            ok("/api/decks?q=a%20b&author=me").as_deref(),
            Some("http://localhost:5173/api/decks?q=a%20b&author=me")
        );
        for path in [
            "/api/admin",
            "/api/admin/reports",
            "/api//admin",
            "/api//admin/reports",
            "/api//device",
            "/api//device/token",
            "/api/.//admin/reports",
            "/api/./admin",
            "/api/./admin/reports",
            "/api/me/",
            "/api/decks//3",
            "/api/",
            "/api/ADMIN/reports",
            "/api/device/token",
            "/api/device",
            "/api/./device/code",
            "/api/x/../admin/reports",
            "/api/../admin",
            "/api/%61dmin/reports",
            "/api/%2e%2e/admin",
            "/api/..\\admin",
            "/api/ad\tmin/reports",
            "/api/de\nvice/token",
            "/api/me#x",
            "/api/me?x#y",
            "/api",
            "/apix/me",
            "api/me",
            "//evil.example/api/me",
            "https://evil.example/api/me",
            "/admin",
            "",
        ] {
            assert_eq!(ok(path), None, "{path}");
        }
        assert!(api_url("https://spam-live.elwqnn.eu", "/api/cards?lang=fr").is_some());
    }

    #[cfg(unix)]
    #[test]
    fn checks_the_appimage_folder() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("spam-live-writable-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        assert!(writable(&dir));
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).unwrap();
        let root = fs::File::create(dir.join("x")).is_ok();
        assert_eq!(writable(&dir), root);
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        fs::remove_dir_all(&dir).unwrap();
        assert!(!writable(&dir));
    }

    #[test]
    fn api_methods() {
        for method in ["GET", "POST", "PUT", "PATCH", "DELETE"] {
            assert_eq!(api_method(method).map(|m| m.to_string()).as_deref(), Some(method));
        }
        for method in ["HEAD", "OPTIONS", "TRACE", "CONNECT", "get", ""] {
            assert!(api_method(method).is_none(), "{method}");
        }
    }

    #[test]
    fn app_urls() {
        let url = |s: &str| Url::parse(s).unwrap();
        assert!(app_url(&url("tauri://localhost/decks/1")));
        assert!(app_url(&url("http://tauri.localhost/settings?link")));
        assert!(app_url(&url("https://tauri.localhost/")));
        assert_eq!(app_url(&url("http://localhost:1420/cards")), cfg!(debug_assertions));
        for other in
            ["https://spam-live.elwqnn.eu/decks", "tauri://evil/", "http://localhost:5173/", "file:///etc/passwd"]
        {
            assert!(!app_url(&url(other)), "{other}");
        }
    }
}
