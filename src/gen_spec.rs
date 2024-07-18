#[cfg(not(feature = "scrape"))]
compile_error!("generating the spec requires the 'scrape' feature");

use serde::Serialize;
use wgsl_spec::scrape;

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn main() -> Result<()> {
    let root_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    let document = std::fs::read_to_string(root_dir.join("upstream/WGSL.html"))?;
    let html = scraper::Html::parse_document(&document);

    write_file(&root_dir.join("spec/tokens.json"), &scrape::extract_tokens(&html))?;
    write_file(&root_dir.join("spec/functions.json"), &scrape::extract_functions(&html))?;

    Ok(())
}

fn write_file(path: &std::path::Path, data: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let output = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(path)?;
    serde_json::to_writer_pretty(output, &data)?;
    Ok(())
}
