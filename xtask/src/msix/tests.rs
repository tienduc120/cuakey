use super::{newest_sdk, package_version, stamp_version};

#[test]
fn a_cargo_version_gains_a_fourth_part_of_zero() {
    assert_eq!(
        package_version("1.0.2").unwrap(),
        "1.0.2.0",
        "the Store wants four parts, and keeps the fourth at zero"
    );
}

#[test]
fn a_pre_release_version_is_refused() {
    assert!(
        package_version("1.1.0-beta.1").is_err(),
        "a package version holds only numbers"
    );
}

#[test]
fn a_part_beyond_what_a_package_version_holds_is_refused() {
    assert!(package_version("1.70000.0").is_err(), "each part fits in 16 bits");
}

#[test]
fn a_version_without_three_parts_is_refused() {
    package_version("1.0").unwrap_err();
}

#[test]
fn the_version_replaces_the_placeholder() {
    let manifest = r#"<Identity Name="X" Version="0.0.0.0" />"#;

    let stamped = stamp_version(manifest, "1.0.2.0").unwrap();

    assert_eq!(stamped, r#"<Identity Name="X" Version="1.0.2.0" />"#);
}

#[test]
fn a_manifest_without_the_placeholder_is_refused() {
    assert!(
        stamp_version(r#"<Identity Version="1.0.0.0" />"#, "1.0.2.0").is_err(),
        "a manifest stamped by hand would ship the wrong version without a word"
    );
}

#[test]
fn the_repository_manifest_takes_the_version() {
    let manifest = include_str!("../../../services/cuakey/desktop/package/AppxManifest.xml");

    let stamped = stamp_version(manifest, "1.0.2.0").unwrap();

    assert!(stamped.contains(r#"Version="1.0.2.0""#));
}

#[test]
fn the_newest_sdk_wins_by_number_not_by_text() {
    let folders = ["10.0.17134.0", "x64", "10.0.26100.0", "10.0.9200.0", "x86"];

    assert_eq!(newest_sdk(folders), Some("10.0.26100.0"));
}

#[test]
fn a_bin_folder_without_a_versioned_sdk_has_none() {
    assert_eq!(newest_sdk(["x64", "x86", "arm64"]), None);
}
