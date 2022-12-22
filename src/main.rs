#[cfg(not(windows))]
use std::os::unix::io::{FromRawFd};
use std::process::{Command, Stdio};
use nix::pty::openpty;

fn main() {
    let pty = openpty(None, None).unwrap();

    let mut builder = {
        let cmd = Command::new(std::env::var("SHELL").expect("could not find default shell from $SHELL"));
        cmd };
    builder.stdin(unsafe { Stdio::from_raw_fd(pty.slave) });
    builder.stderr(unsafe { Stdio::from_raw_fd(pty.slave) });
    builder.stdout(unsafe { Stdio::from_raw_fd(pty.slave) });
    builder.spawn().ok();

    //just waits 3 seconds for shell to print some stuff to stdout
    use std::{thread, time};
    let millis = time::Duration::from_millis(2000);
    thread::sleep(millis);
    
    //setup read buffers
    let mut read_buffer = [0;65536];
    nix::unistd::read(pty.master, &mut read_buffer).ok();
    //let mut string_buffer = vec![];

    //for byte in read_buffer.to_vec() {
    //    if byte == 0 {
    //        continue
    //    }
    //    string_buffer.push(byte);
    //}


    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(&read_buffer);
    println!("{:?}", String::from_utf8(parser.screen().state_formatted()));
    println!("{:?}", parser.screen().contents());
    println!("{:?}", parser.screen().size());
    //println!("{:?}",String::from_utf8(string_buffer).unwrap());
}