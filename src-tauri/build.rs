fn main() {
    println!("cargo:rerun-if-changed=windows-app-manifest.xml");

    let profile = std::env::var("PROFILE").unwrap_or_default();
    let attributes = if profile == "release" {
        let windows =
            tauri_build::WindowsAttributes::new().app_manifest(include_str!("windows-app-manifest.xml"));
        tauri_build::Attributes::new().windows_attributes(windows)
    } else {
        tauri_build::Attributes::new()
    };

    tauri_build::try_build(attributes).expect("failed to run tauri build script");
}
