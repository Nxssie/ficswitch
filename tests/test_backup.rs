/// Regression tests for ACF trailing newline preservation in set_betakey_in_acf.
/// These test the public switch_branch function indirectly via a temp file,
/// and the private function behaviour is exercised via round-trip checks.

use std::fs;
use std::path::Path;

// We test via the public API: write a temp ACF file, call switch_branch, read it back.
use ficswitch::core::steam;

fn write_temp_acf(dir: &Path, filename: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(filename);
    fs::write(&path, content).expect("failed to write temp ACF");
    path
}

const SAMPLE_ACF_WITH_NEWLINE: &str = r#""AppState"
{
	"appid"		"526870"
	"Universe"		"1"
	"name"		"Satisfactory"
	"installdir"		"Satisfactory"
	"UserConfig"
	{
		"betakey"		"experimental"
	}
	"MountedConfig"
	{
		"betakey"		"experimental"
	}
}
"#;

const SAMPLE_ACF_WITHOUT_NEWLINE: &str = r#""AppState"
{
	"appid"		"526870"
	"Universe"		"1"
	"name"		"Satisfactory"
	"installdir"		"Satisfactory"
	"UserConfig"
	{
		"betakey"		"experimental"
	}
	"MountedConfig"
	{
		"betakey"		"experimental"
	}
}"#;

#[test]
fn test_acf_trailing_newline_preserved() {
    let dir = std::env::temp_dir().join("ficswitch_test_acf_newline");
    fs::create_dir_all(&dir).unwrap();

    let path = write_temp_acf(&dir, "appmanifest_526870.acf", SAMPLE_ACF_WITH_NEWLINE);

    steam::switch_branch(&path, &steam::Branch::Stable).expect("switch_branch failed");

    let result = fs::read_to_string(&path).expect("failed to read result");
    assert!(
        result.ends_with('\n'),
        "Expected trailing newline to be preserved, but file does not end with '\\n'. \
         Last 10 chars: {:?}",
        &result[result.len().saturating_sub(10)..]
    );

    fs::remove_file(&path).ok();
    fs::remove_dir(&dir).ok();
}

#[test]
fn test_acf_no_trailing_newline_not_added() {
    let dir = std::env::temp_dir().join("ficswitch_test_acf_no_newline");
    fs::create_dir_all(&dir).unwrap();

    let path = write_temp_acf(&dir, "appmanifest_526870.acf", SAMPLE_ACF_WITHOUT_NEWLINE);

    steam::switch_branch(&path, &steam::Branch::Stable).expect("switch_branch failed");

    let result = fs::read_to_string(&path).expect("failed to read result");
    assert!(
        !result.ends_with('\n'),
        "Expected no trailing newline, but file ends with '\\n'. \
         Last 10 chars: {:?}",
        &result[result.len().saturating_sub(10)..]
    );

    fs::remove_file(&path).ok();
    fs::remove_dir(&dir).ok();
}
