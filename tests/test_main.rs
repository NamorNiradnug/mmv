mod common;
use std::{fs::read_dir, iter::empty, path::PathBuf, process::Command};

use common::generate_files;

use assert_cmd::prelude::*;
use predicates::str::contains;

#[test]
fn test_main_simple() -> anyhow::Result<()> {
    let temporary_directory = generate_files(
        ["Harry", "Ron", "Hermy", "Neville"]
            .iter()
            .map(|name| PathBuf::from("griffindor").join(name)),
        [PathBuf::from("expelled/")].into_iter(),
    )?;
    let mut mmv = Command::cargo_bin("mmv")?;
    mmv.current_dir(temporary_directory.path())
        .args(["griffindor/H*y", "expelled/H#1y-#1"])
        .assert()
        .success()
        .stdout(contains("->").count(2))
        .stdout(contains("Done").count(2));
    let expelled = temporary_directory.path().join("expelled/");
    assert!(expelled.join("Harry-arr").exists());
    assert!(expelled.join("Hermy-erm").exists());
    Ok(temporary_directory.close()?)
}

#[test]
fn test_into_same_directory() -> anyhow::Result<()> {
    let temporary_directory = generate_files(
        ["Harry", "Ron", "Hermy", "Neville", "Hagrid"]
            .map(PathBuf::from)
            .into_iter(),
        empty(),
    )?;
    let mut mmv = Command::cargo_bin("mmv")?;
    mmv.current_dir(temporary_directory.path())
        .args(["H*", "H#1-starts-with-H"])
        .assert()
        .success()
        .stdout(contains("Done").count(3));
    Ok(temporary_directory.close()?)
}

#[test]
fn test_no_target_directory() -> anyhow::Result<()> {
    let temporary_directory = generate_files(
        ["Harry", "Ron", "Hermy", "Neville"]
            .iter()
            .map(|name| PathBuf::from("griffindor").join(name)),
        empty(),
    )?;
    let mut mmv = Command::cargo_bin("mmv")?;
    mmv.current_dir(temporary_directory.path().join("griffindor"))
        .args(["Neville", "hufflepuff/#1-is-dumb-sometimes"])
        .assert()
        .code(1)
        .stderr(contains("Target directory \"hufflepuff/\" doesn't exist"));
    Ok(temporary_directory.close()?)
}

#[test]
fn test_no_matches_for_pattern() -> anyhow::Result<()> {
    let temporary_directory = generate_files(
        ["Harry", "Ron", "Hermy", "Neville"]
            .iter()
            .map(|name| PathBuf::from("griffindor").join(name)),
        empty(),
    )?;
    let mut mmv = Command::cargo_bin("mmv")?;
    mmv.current_dir(temporary_directory.path().join("griffindor"))
        .args(["A*", "../A#1"])
        .assert()
        .failure()
        .stderr(contains("No files matching pattern"));
    Ok(temporary_directory.close()?)
}

#[test]
fn test_mmv_doesnt_overwrite_existing_file() -> anyhow::Result<()> {
    let temporary_directory = generate_files(
        ["Harry", "Ron", "Hermy", "Neville"]
            .iter()
            .map(|name| PathBuf::from("griffindor").join(name))
            .chain(
                ["Draco, Crabbe", "Goyle", "Harry"]
                    .iter()
                    .map(|name| PathBuf::from("slytherin").join(name)),
            ),
        empty(),
    )?;
    let mut mmv = Command::cargo_bin("mmv")?;
    mmv.current_dir(temporary_directory.path().join("griffindor"))
        .args(["H*y", "../slytherin/H#1y"])
        .assert()
        .success()
        .stdout(contains("Skip").count(1))
        .stdout(contains("Done").count(1));
    assert!(temporary_directory.path().join("griffindor/Harry").exists());
    assert!(!temporary_directory.path().join("griffindor/Hermy").exists());
    assert!(temporary_directory.path().join("slytherin/Hermy").exists());
    assert!(temporary_directory.path().join("slytherin/Harry").exists());
    Ok(temporary_directory.close()?)
}

#[test]
fn test_force_flag() -> anyhow::Result<()> {
    let temporary_directory = generate_files(
        (1..20)
            .map(|photo_index| PathBuf::from("png_images").join(photo_index.to_string() + ".png"))
            .chain((10..30).map(|photo_index| {
                PathBuf::from("jpg_images").join(photo_index.to_string() + ".jpg")
            })),
        empty(),
    )?;
    let mut mmv = Command::cargo_bin("mmv")?;
    mmv.current_dir(temporary_directory.path())
        .args(["-f", "png_images/*.png", "jpg_images/#1.jpg"])
        .assert()
        .success()
        .stdout(contains("Done").count(19));
    assert_eq!(
        read_dir(temporary_directory.path().join("png_images"))?.count(),
        0
    );
    for photo_index in 1..30 {
        assert!(temporary_directory
            .path()
            .join("jpg_images")
            .join(photo_index.to_string() + ".jpg")
            .exists());
    }
    Ok(temporary_directory.close()?)
}
