use nexus::AddonFlags;

pub mod addon;

nexus::export! {
    name: "Gw2 Executable Runner",
    signature: -0x7A8B9C2A,
    load: addon::load,
    unload: addon::unload,
    flags: AddonFlags::None,
    provider: nexus::UpdateProvider::GitHub,
    update_link: "https://github.com/OlivierFortier/gw2-executable-runner",
    log_filter: "trace"
}
