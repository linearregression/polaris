extern crate time;

use log::{Logger,LogRecord};
use time::{now,strftime};

use std::io::net::ip::{SocketAddr};
use std::io::{File, Open, Write,USER_RWX};
use std::os::tmpdir;
use std::io::fs::{mkdir_recursive,PathExtensions};
use std::sync::mpsc::{channel,Sender,Receiver};
use std::thread::Thread;

// based on http://joshitech.blogspot.com/2014/12/rust-customer-logger.html
pub struct LocalLogger {
    sender: Sender<String>,
}
impl LocalLogger {
    pub fn new(sender: Sender<String>) -> LocalLogger {
        LocalLogger { sender: sender }
    }
}
impl Logger for LocalLogger {
    fn log(&mut self, record: &LogRecord) {
        let entry = format!(
                       "{} {} {}:{} (line {}) {}",
                       time::strftime("%Y-%m-%d %H:%M:%S.%f %Z", &time::now()).unwrap(),
                       record.level,
                       record.module_path,
                       record.file,
                       record.line,
                       record.args);

        match self.sender.send(entry) {
            Ok(()) => {}
            Err(e) => println!("failed to log"),
        }
    }  
}

fn log_listen(socket: &SocketAddr) -> Sender<String> {
    let mut p = tmpdir().clone();
    p.push("polaris");
    if !p.exists() {
        match mkdir_recursive(&p, USER_RWX) {
            Ok(_) => {}
            Err(e) => panic!("failed to create tmp dir: {}", e),
        };
    }
    
    p.push(format!("{}", socket));
    let mut file = match File::open_mode(&p, Open, Write) {
        Ok(f) => f,
        Err(e) => panic!("file error: {}", e),
    };
    
    let (sender, receiver) = channel::<String>();
    
    Thread::spawn(move || {
        // TODO: try_recv() does *not* block, and might be nice for a gentle shutdown of the listener
        loop {
            let res = receiver.recv();
            if res.is_ok() {
                file.write_line(res.unwrap().as_slice()).ok().expect("failed to log!");
            }
        }
    });
    sender
}

