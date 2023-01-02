use clap::Parser;
use nix::{
    sys::memfd::{memfd_create, MemFdCreateFlag},
    unistd::fexecve,
};
use std::{
    ffi::CString,
    fs::File,
    io::{Read, Write},
    os::fd::FromRawFd,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of the file to execute
    #[arg(short, long)]
    path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut bin_file = File::open(args.path.as_str())?;
    let mut buf = vec![0; 10 * 1024 * 1024];

    let l = bin_file.read(&mut buf)?;

    // println!("File size: {} bytes", l);

    let raw_fd = memfd_create(
        CString::new("memfd-demo")?.as_c_str(),
        MemFdCreateFlag::MFD_CLOEXEC,
    )?;

    // println!("RawFD: {}", raw_fd);

    let mut file = unsafe { File::from_raw_fd(raw_fd) };

    file.write_all(&buf[..l])?;

    let args: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).unwrap())
        .collect();

    let vars: Vec<CString> = std::env::vars()
        .map(|(mut var, value)| {
            var.extend(['='].iter());
            var.extend(value.chars());

            // println!("ENV: {}", &var);

            CString::new(var).unwrap()
        })
        .collect();

    if let Err(errno) = fexecve(raw_fd, args.as_slice(), &vars) {
        println!("fexecve error: {:?}", errno);
    }

    Ok(())
}
