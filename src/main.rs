use nix::pty;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use nix::unistd::read;
use std::process::Command;
fn main() {
    let default_shell = std::env::var("SHELL")
        .expect("could not find default shell from $SHELL");
    let a = pty::openpty(None, None).unwrap(); 
    let master = a.master;
    let slave = a.slave;
    Command::new(&default_shell)
                .spawn()
                .expect("failed to spawn");
    std::thread::sleep(std::time::Duration::from_millis(2000));
    std::process::exit(0);
    let mut read_buffer = [0; 65536];
    let read_result = read(master, &mut read_buffer);
    let result;
    match read_result {
        Ok(bytes_read) => { result = Some(read_buffer[..bytes_read].to_vec()); },
        Err(_e) => { result = None; }
    }
    println!("{:?}, {}, {}", a, master, slave);
    std::process::exit(0);
}
