use eframe::egui;
use std::fs::File;
use nix::errno::Errno;
use nix::pty::{forkpty, ForkptyResult};
use std::ffi::{CStr, CString};
use std::os::fd::{OwnedFd, RawFd, AsRawFd};
use std::io::Read;
use nix::unistd::execvp;

fn main() {
    let fd =
        unsafe{ let res = forkpty(None, None).expect("forkpty() failed");
                     match res {
                         ForkptyResult::Parent { master, .. }  => {
                             println!("I am your father.");
                             master
                         },
                         ForkptyResult::Child =>{
                             println!("Luke, I am your...child?");
                             let shell = CStr::from_bytes_with_nul(b"sh\0").expect("Should always have null terminator on execvp");
                             let args: &[&[u8]] = &[b"sh\0", b"-c\0", b"echo Hello Paul\0"];
                             let args: Vec<&'static CStr> = args
                                .iter()
                                .map(|v| CStr::from_bytes_with_nul(v).expect("Should alweays have null terminator on args"))
                                .collect::<Vec<_>>();
                             execvp(shell, &args).unwrap();
                             return
                         },
                     }
    };

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "PaulTerm",
        native_options,
        Box::new(move |cc| Ok(Box::new(PaulTermGui::new(cc, fd)))));
}

struct PaulTermGui {
    buf: Vec<u8>,
    fd: OwnedFd
}

impl PaulTermGui {
    fn new(cc: &eframe::CreationContext<'_>, fd: OwnedFd) -> Self {
        let flags_as_bits = nix::fcntl::fcntl(fd.as_raw_fd(), nix::fcntl::FcntlArg::F_GETFL)
            .expect("Getting file descriptor flags failed...");
        let mut flags_from_bits = nix::fcntl::OFlag::from_bits(flags_as_bits & nix::fcntl::OFlag::O_ACCMODE.bits())
            .expect("Converting file descriptor access mode flags from bits failed...");
        flags_from_bits.set(nix::fcntl::OFlag::O_NONBLOCK, true);

        nix::fcntl::fcntl(fd.as_raw_fd(), nix::fcntl::FcntlArg::F_SETFL(flags_from_bits)).expect("Setting file descriptor to non-blocking failed...");
        PaulTermGui {
            buf: Vec::new(),
            fd: fd.into(),
        }
    }
}

impl eframe::App for PaulTermGui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut buf = vec![0u8; 4096];
        match nix::unistd::read(self.fd.as_raw_fd(), &mut buf) {
            Ok(read_size) => {
                self.buf.extend_from_slice(&buf[0..read_size]);
            }
            Err(e) => {
                if e != Errno::EAGAIN {
                    println!("Failure: {e}");
                    }
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            
            unsafe {ui.label(std::str::from_utf8_unchecked(&self.buf))}
        });
    }
}

