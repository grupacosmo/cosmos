fn main() -> eyre::Result<()> {
    // read env variables that were set in build script
    let bios_path = env!("BIOS_PATH");
    println!("{bios_path}");

    std::process::Command::new("qemu-system-x86_64")
        .arg("-drive")
        .arg(format!("format=raw,file={bios_path}"))
        .spawn()?
        .wait()?;

    Ok(())
}
