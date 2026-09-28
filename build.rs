#[cfg(target_os = "windows")]
fn main() {
    use std::env;
    use std::fs::File;
    use std::path::PathBuf;

    const SOURCE_ICON: &str = "assets/code-bundler-icon.png";

    println!("cargo:rerun-if-changed={SOURCE_ICON}");

    let source = image::open(SOURCE_ICON)
        .expect("the Code Bundler PNG icon must be readable")
        .into_rgba8();
    let mut directory = ico::IconDir::new(ico::ResourceType::Icon);

    for size in [16, 24, 32, 48, 64, 128, 256] {
        let resized =
            image::imageops::resize(&source, size, size, image::imageops::FilterType::Lanczos3);
        let icon = ico::IconImage::from_rgba_data(size, size, resized.into_raw());
        directory.add_entry(
            ico::IconDirEntry::encode(&icon).expect("the Windows icon frame must encode"),
        );
    }

    let icon_path = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be available"))
        .join("code-bundler.ico");
    let mut icon_file = File::create(&icon_path).expect("the generated ICO file must be writable");
    directory
        .write(&mut icon_file)
        .expect("the generated ICO file must be valid");

    winresource::WindowsResource::new()
        .set_icon(
            icon_path
                .to_str()
                .expect("the ICO path must be valid UTF-8"),
        )
        .set("ProductName", "Code Bundler")
        .set("FileDescription", "Code Bundler")
        .set("LegalCopyright", "Copyright (c) Code Bundler contributors")
        .compile()
        .expect("the Windows application resources must compile");
}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("cargo:rerun-if-changed=assets/code-bundler-icon.png");
}
