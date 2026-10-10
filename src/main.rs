use std::net::{TcpStream, SocketAddr};
use std::time::Duration;
use std::thread;
fn scan_port(ip: &str, port: u16) -> bool {
    let addr: SocketAddr = match format!("{}:{}", ip, port).parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()
}

fn main() {
    let ip = "127.0.0.1";
    let ports= [22, 80, 443, 445, 8080];
    // Tạo danh sahcs để dùng cho các thread
    let mut handles = vec![];
    for  port in ports {
        let ip = ip.to_string();
        let handle = thread::spawn(move || {
            let open = scan_port(&ip, port);
            (port, open)
        });
        handles.push(handle);
    }
    for handle in handles {
        let (port, open) = handle.join().unwrap();
        if open {
            println!("Port {} OPEN", port);
        } else {
            println!("Port {} closed", port);
        }
    }
}