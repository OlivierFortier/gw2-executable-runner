use nexus::AddonFlags;

pub mod addon;

nexus::export! {
    name: "Gw2 Executable Loader",
    signature: -0x7A8B9C2A,
    load: addon::load,
    unload: addon::unload,
    flags: AddonFlags::IsVolatile,
    provider: nexus::UpdateProvider::None,
    log_filter: "trace"
}
