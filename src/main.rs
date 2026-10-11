use clap::Parser;
use std::net::{SocketAddr, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Parser)]
struct Args {
    /// IP cần quét
    ip: String,
    /// Port bắt đầu
    #[arg(short, long, default_value_t = 1)]
    start: u16,
    /// Port kết thúc
    #[arg(short, long, default_value_t = 1024)]
    end: u16,
    /// Số thread chạy song song
    #[arg(short, long, default_value_t = 100)]
    workers: usize,
}

fn scan_port(ip: &str, port: u16) -> bool {
    let addr: SocketAddr = match format!("{}:{}", ip, port).parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok()
}

fn main() {
    let args = Args::parse();

    // Kênh gửi port cần quét tới các worker
    let (job_tx, job_rx) = mpsc::channel::<u16>();
    // Kênh gửi kết quả về main
    let (result_tx, result_rx) = mpsc::channel::<(u16, bool)>();

    let job_rx = Arc::new(Mutex::new(job_rx));
    let ip = Arc::new(args.ip.clone());

    let mut handles = vec![];

    // Tạo N worker
    for _ in 0..args.workers {
        let job_rx = Arc::clone(&job_rx);
        let result_tx = result_tx.clone();
        let ip = Arc::clone(&ip);

        let handle = thread::spawn(move || {
            loop {
                // Lấy 1 port từ hàng đợi (khóa Mutex trong lúc lấy)
                let port = {
                    let rx = job_rx.lock().unwrap();
                    match rx.recv() {
                        Ok(p) => p,
                        Err(_) => break, // kênh đóng → hết việc → thoát
                    }
                };
                let open = scan_port(&ip, port);
                result_tx.send((port, open)).unwrap();
            }
        });
        handles.push(handle);
    }

    // main đóng result_tx gốc để worker clone cuối cùng vẫn giữ
    drop(result_tx);

    // Gửi việc vào hàng đợi
    for port in args.start..=args.end {
        job_tx.send(port).unwrap();
    }
    drop(job_tx); // đóng job_tx → worker nhận Err → thoát vòng lặp

    // Thu kết quả
    let mut open_ports = vec![];
    for (port, open) in result_rx {
        if open {
            open_ports.push(port);
        }
    }

    // Chờ worker xong
    for h in handles {
        h.join().unwrap();
    }

    // In kết quả đã sort
    open_ports.sort();
    for port in open_ports {
        println!("Port {} OPEN", port);
    }
}