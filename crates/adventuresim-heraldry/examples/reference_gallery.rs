//! Rebuild a local, reproducible clean/painterly comparison from the public recipes.
use adventuresim_heraldry::{
    artwork::Artwork,
    bake::{Baked, Resolution},
    document::PaintedModeling,
    export, presets,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/heraldry/gallery".into());
    std::fs::create_dir_all(&directory)?;
    let mut html = String::from(
        "<!doctype html><meta charset=utf-8><title>Heraldry reference studies</title><style>body{background:#152028;color:#eadcc3;font:16px system-ui;margin:32px}main{display:grid;grid-template-columns:repeat(3,1fr);gap:30px}img{width:100%;max-width:260px}figure{margin:0}section{display:flex;gap:10px}h2{font-size:16px}a{color:inherit}</style><h1>Heraldry · clean and painterly studies</h1><p>Shared parametric families; refer to REFERENCES.md for the historical sources and limits.</p><main>",
    );
    for name in presets::PRESETS {
        let mut d = presets::preset(name)?;
        html.push_str(&format!("<figure><h2>{name}</h2><section>"));
        for (label, paint) in [
            ("clean", PaintedModeling::FLAT),
            ("painterly", PaintedModeling::MODELED),
        ] {
            d.drawing.painted_modeling = paint;
            let stem = format!("{name}-{label}");
            let prefix = format!("{directory}/{stem}");
            let b = Baked::generate(&d, Resolution::Preview)?;
            std::fs::write(format!("{prefix}.json"), d.to_json()?)?;
            std::fs::write(
                format!("{prefix}.svg"),
                Artwork::compose(&d)?.svg(&d.surface.palette),
            )?;
            std::fs::write(format!("{prefix}.png"), export::png(&b.flat, b.size)?)?;
            html.push_str(&format!(
                "<a href='{stem}.json'><img src='{stem}.png' alt='{name} {label}' style='aspect-ratio:{}/{}'></a>",
                d.surface.width.0, d.surface.height.0,
            ));
        }
        html.push_str("</section></figure>");
    }
    html.push_str("</main><p>Lion artwork adapted from <a href='https://commons.wikimedia.org/wiki/File:Lion_Rampant_Or_(16th_century_German).svg'>German lion by Tom-L after Rinaldum</a>, <a href='https://creativecommons.org/licenses/by-sa/3.0/'>CC BY-SA 3.0</a>. Modified proportions, tinctures, painted tones and line widths.</p>");
    std::fs::write(format!("{directory}/index.html"), html)?;
    Ok(())
}
