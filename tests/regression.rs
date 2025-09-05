use sitegen::SiteGenerator;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::Builder;

#[test]
fn test_render_site() {
    // Set TEST_TEMP_RETAIN env var to retain generated output
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/fixtures");
    let site = root.join("tests/site");
    let retain = std::env::var("TEST_TEMP_RETAIN").is_ok();
    let output = Builder::new()
        .disable_cleanup(retain)
        .tempdir()
        .expect("tempdir failed")
        .path()
        .to_owned();
    if retain {
        eprintln!("TEMPDIR {}", output.display());
    }
    let generator = SiteGenerator::new(&site).expect("SiteGenerator error");
    generator.build(&output).expect("build failed");

    compare(&fixtures.join("index.html"), &output.join("index.html"));
    compare(&fixtures.join("style.css"), &output.join("style.css"));
}

fn compare(fixture: &Path, actual: &Path) {
    assert_eq!(
        fs::read_to_string(fixture)
            .unwrap_or_else(|err| panic!("failed to read {} - {err:?}", fixture.display())),
        fs::read_to_string(actual)
            .unwrap_or_else(|err| panic!("failed to read {} - {err:?}", actual.display()))
    );
}
