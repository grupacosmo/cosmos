use bootloader::{BootConfig, DiskImageBuilder};
use std::{
    io,
    path::Path,
    process::{Command, Stdio},
    thread,
};

/// Creates a test case that runs a binary form `test_kernel` crate in qemu.
#[macro_export]
macro_rules! test {
    ($bin_name:ident) => {
        #[test]
        fn $bin_name() {
            $crate::runner::run(env!(concat!(
                "CARGO_BIN_FILE_TEST_KERNEL_",
                stringify!($bin_name)
            )));
        }
    };
}

pub fn run(path: &str) {
    let path = Path::new(path);
    let mut image_builder = DiskImageBuilder::new(path.to_path_buf());
    let image_path = path.with_extension(".mbr");
    image_builder
        .set_boot_config(&{
            let mut boot_config = BootConfig::default();
            boot_config.serial_logging = false;
            boot_config
        })
        .create_bios_image(&image_path)
        .unwrap();

    let mut child = Command::new("qemu-system-x86_64")
        .arg("-drive")
        .arg(format!("format=raw,file={}", image_path.display()))
        .arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04")
        .arg("-serial")
        .arg("stdio")
        .arg("-display")
        .arg("none")
        .arg("--no-reboot")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .unwrap();

    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();

    let t1 = thread::spawn(move || io::copy(&mut child_stdout, &mut io::stdout()));
    let t2 = thread::spawn(move || io::copy(&mut child_stderr, &mut io::stderr()));

    let status = child.wait().unwrap();
    match status.code() {
        Some(33) => {}
        Some(35) => panic!("Test failed"),
        other => panic!("Test failed with unexpected exit code {other:?}"),
    }

    t1.join().unwrap().unwrap();
    t2.join().unwrap().unwrap();
}
