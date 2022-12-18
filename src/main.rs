use core::panic;
#[cfg(not(windows))]
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::process::{Child, Command, Stdio};
use nix::pty::{openpty, OpenptyResult};
use std::io::{Error, ErrorKind, Result};
use libc::{self, c_int, winsize, TIOCSCTTY};
use nix::sys::termios::{self, InputFlags, SetArg};
use log::error;
use std::os::unix::process::CommandExt;
use std::fs::File;
//use signal_hook_mio::v0_6::Signals;
//use signal_hook::consts as sigconsts;

macro_rules! die {
    ($($arg:tt)*) => {{
        error!($($arg)*);
        std::process::exit(1);
    }}
}

struct PtyPair {
    pub master: RawFd,
    pub slave: RawFd,
}

#[derive(Debug)]
pub struct Pty {
    child: Child,
    file: File,
    //token: mio::Token,
    //signals: Signals,
    //signals_token: mio::Token,
}

#[derive(Copy, Clone, Debug)]
pub struct WindowSize {
    pub num_lines: u16,
    pub num_cols: u16,
    pub cell_width: u16,
    pub cell_height: u16,
}

/// Types that can produce a `libc::winsize


fn main() {
    let pty = spawn_pty().unwrap();
    if let Ok(mut termios) = termios::tcgetattr(pty.master) {
        // Set character encoding to UTF-8.
        termios.input_flags.set(InputFlags::IUTF8, true);
        let _ = termios::tcsetattr(pty.master, SetArg::TCSANOW, &termios);
    }
    let mut buf = [0; 1024];
    let shell = std::env::var("SHELL").expect("could not find default shell from $SHELL");
    let mut builder = {
        let mut cmd = Command::new(shell);
        cmd };
    builder.stdin(unsafe { Stdio::from_raw_fd(pty.slave) });
    builder.stderr(unsafe { Stdio::from_raw_fd(pty.slave) });
    builder.stdout(unsafe { Stdio::from_raw_fd(pty.slave) });
    println!("{:?}", pty);
    //loop {}

    unsafe {
        builder.pre_exec(move || {
            // Create a new process group.
            let err = libc::setsid();
            if err == -1 {
                return Err(Error::new(ErrorKind::Other, "Failed to set session id"));
            }

            set_controlling_terminal(pty.slave);

            // No longer need slave/master fds.
            libc::close(pty.slave);
            libc::close(pty.master);

            libc::signal(libc::SIGCHLD, libc::SIG_DFL);
            libc::signal(libc::SIGHUP, libc::SIG_DFL);
            libc::signal(libc::SIGINT, libc::SIG_DFL);
            libc::signal(libc::SIGQUIT, libc::SIG_DFL);
            libc::signal(libc::SIGTERM, libc::SIG_DFL);
            libc::signal(libc::SIGALRM, libc::SIG_DFL);

            Ok(())
        });
    }
    //let signals = Signals::new([sigconsts::SIGCHLD]).expect("error preparing signal handling");


    match builder.spawn() {
        Ok(child) => {
            unsafe {
                // Maybe this should be done outside of this function so nonblocking
                // isn't forced upon consumers. Although maybe it should be?
                set_nonblocking(pty.master);
            }

            let mut ptynew = Pty {
                child,
                file: unsafe { File::from_raw_fd(pty.master) },
                //token: mio::Token::from(0),
                //signals,
                //signals_token: mio::Token::from(0),
            };
            println!("{:?}", ptynew);
            //ptynew.on_resize(window_size);
        },
        Err(err) => { panic!("bah") }
    }
}


fn spawn_pty() -> Result<OpenptyResult> {
    //let mut window_size = size;
    //window_size.ws_xpixel = 0;
    //window_size.ws_ypixel = 0;
    Ok(openpty(None, None)?)
}

/// Really only needed on BSD, but should be fine elsewhere.
fn set_controlling_terminal(fd: c_int) {
    let res = unsafe {
        // TIOSCTTY changes based on platform and the `ioctl` call is different
        // based on architecture (32/64). So a generic cast is used to make sure
        // there are no issues. To allow such a generic cast the clippy warning
        // is disabled.
        #[allow(clippy::cast_lossless)]
        libc::ioctl(fd, TIOCSCTTY as _, 0)
    };

    if res < 0 {
        die!("ioctl TIOCSCTTY failed: {}", Error::last_os_error());
    }
}

unsafe fn set_nonblocking(fd: c_int) {
    use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

    let res = fcntl(fd, F_SETFL, fcntl(fd, F_GETFL, 0) | O_NONBLOCK);
    assert_eq!(res, 0);
}