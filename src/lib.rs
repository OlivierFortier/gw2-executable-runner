use nexus;


nexus::export! {
    name: "Gw2 Binary Loader",
    signature: -0x7A8B9C2V,
    load: nexus_addon::nexus_load,
    unload: nexus_addon::nexus_unload,
    flags: AddonFlags::IsVolatile,
    provider: nexus::UpdateProvider::None,
    log_filter: "trace"
}

pub fn vahoom() {
 format!("yee")
}