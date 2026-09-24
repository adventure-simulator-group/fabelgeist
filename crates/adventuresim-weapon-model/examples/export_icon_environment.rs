//! Write the shared studio environment for offline equipment portrait baking.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: export_icon_environment <output.hdr>")?;
    std::fs::write(path, adventuresim_weapon_model::environment_hdr())?;
    Ok(())
}
