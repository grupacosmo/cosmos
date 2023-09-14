fn main() -> eyre::Result<()> {
    // read env variables that were set in build script
    let bios_path = env!("BIOS_PATH");
    let cargo_dir = env!("CARGO_MANIFEST_DIR");
    std::process::Command::new("qemu-system-x86_64")
        .arg("-drive")
        .arg(format!("format=raw,file={bios_path}"))
        .arg("-boot")
        .arg( format!("menu=on,splash={}\\src\\assets\\bootsplash.bmp,splash-time=1000", cargo_dir))
        .spawn()?
        .wait()?;

    Ok(())
}
