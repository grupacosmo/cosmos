fn main() -> eyre::Result<()> {
    // read env variables that were set in build script
    let bios_path = env!("BIOS_PATH");
    let cargo_dir = env!("CARGO_MANIFEST_DIR");
    let bootsplash_path = format!("{cargo_dir}/src/assets/bootsplash.bmp");
    std::process::Command::new("qemu-system-x86_64")
        .arg("-drive")
        .arg(format!("format=raw,file={bios_path}"))
        .arg("-boot")
        .arg(bootsplash_path)
        .spawn()?
        .wait()?;

    Ok(())
}
