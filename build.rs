fn main() {
    println!("cargo:rerun-if-changed=assets/motaa-desktop.rc");
    println!("cargo:rerun-if-changed=assets/app-icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/motaa-desktop.rc", embed_resource::NONE)
            .manifest_required()
            .expect("embed Windows application icon resource");
    }
}
