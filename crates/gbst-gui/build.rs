#[cfg(target_os = "windows")]
fn main() {
    let mut resource = winresource::WindowsResource::new();
    resource.set_icon("assets/icon.ico");
    resource
        .compile()
        .expect("failed to embed the GBST Windows application icon");
}

#[cfg(not(target_os = "windows"))]
fn main() {}
