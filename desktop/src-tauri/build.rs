fn main() {
    println!("cargo:rerun-if-env-changed=SPAM_SITE_URL");
    let release = std::env::var("PROFILE").is_ok_and(|p| p == "release");
    if release && !std::env::var("SPAM_SITE_URL").is_ok_and(|url| url.starts_with("https://")) {
        panic!("SPAM_SITE_URL must start with https:// in a release build");
    }
    tauri_build::build()
}
