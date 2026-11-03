#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod uploader;

use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{
        Mutex, MutexGuard, PoisonError,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use reqwest::Url;
use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Emitter, Manager, WindowEvent, Wry,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_autostart::ManagerExt;

use uploader::{Api, RETRY, Report, Uploader};

const SITE: &str = match option_env!("SPAM_SITE_URL") {
    Some(url) => url,
    None => "http://localhost:5173",
};
const RELEASES: &str = "https://github.com/elwqnn/spam-live/releases/latest";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const KEYRING_SERVICE: &str = "io.github.elwqnn.spamlive";
const POLL: Duration = Duration::from_secs(2);
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
    reset: bool,
}

struct Tray(MenuItem<Wry>, MenuItem<Wry>);

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
    autostart: bool,
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

#[tauri::command(async)]
fn unlink(app: AppHandle) -> Result<(), String> {
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
fn tray_labels(app: AppHandle, open: String, quit: String) {
    if let Some(tray) = app.try_state::<Tray>() {
        let _ = tray.0.set_text(open);
        let _ = tray.1.set_text(quit);
    }
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
    loop {
        let now = Instant::now();
        if now >= version_check {
            version_check = now + RETRY;
            if let Ok(min) = api.min_version() {
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
                {
                    let mut d = data(&app);
                    if reports.iter().any(Report::unauthorized) && d.token.as_ref() == Some(&token) {
                        let _ = keyring().and_then(|entry| entry.delete_credential());
                        d.token = None;
                        d.notice = Some(Notice::Revoked);
                    }
                    d.last = Some(last);
                }
                emit(&app);
            }
        }
        thread::sleep(POLL);
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show(app)))
        .plugin(tauri_plugin_autostart::Builder::new().arg("--minimized").build())
        .manage(Shared::default())
        .invoke_handler(tauri::generate_handler![
            get_status,
            pair,
            cancel_pair,
            unlink,
            set_folder,
            set_autostart,
            open,
            tray_labels
        ])
        .setup(|app| {
            let settings: Settings = settings_file(app.handle())
                .and_then(|file| fs::read(file).ok())
                .and_then(|json| serde_json::from_slice(&json).ok())
                .unwrap_or_default();
            apply_folder(app.handle(), settings.folder);
            let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let mut tray = TrayIconBuilder::new()
                .tooltip("SPAM Live")
                .menu(&Menu::with_items(app, &[&open, &quit])?)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show(app),
                    "quit" => app.exit(0),
                    _ => {}
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            app.manage(Tray(open, quit));
            if !std::env::args().any(|arg| arg == "--minimized") {
                show(app.handle());
            }
            let handle = app.handle().clone();
            thread::spawn(move || worker(handle));
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
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
}
