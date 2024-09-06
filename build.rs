use rustc_version::{version_meta, Channel, VersionMeta};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(CHANNEL_NIGHTLY)");
    if matches!(version_meta(), Ok(VersionMeta { channel: Channel::Nightly, .. })) {
        println!("cargo:rustc-cfg=CHANNEL_NIGHTLY");
    }
}
