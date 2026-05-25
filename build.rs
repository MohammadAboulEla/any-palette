fn main() {
    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile()
            .expect("failed to embed Windows icon");
        println!("cargo:rerun-if-changed=assets/icon.ico");
    }
}
