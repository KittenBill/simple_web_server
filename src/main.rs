
use std::{
    fs::read_to_string, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, sync::mpsc::{self, Receiver, Sender}, thread, time::Duration
};

use simple_web_server::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let N = 4;
    let number_of_stream_to_process = 10;

    let pool = ThreadPool::new(N);
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    for stream in listener.incoming().take(number_of_stream_to_process) {
        let stream = stream.unwrap();
        let shutdown_tx = shutdown_tx.clone();

        pool.execute(|| {
            handle_connection(stream, shutdown_tx);
        });

        if let Ok(message) = shutdown_rx.try_recv() {
            println!("Web Server shutting down due to {}", message);
            break;
        }
    }

    drop(pool);
    println!("Web server has gracefully been shutdown. Good bye.")
}

fn handle_connection(mut stream: TcpStream, shutdown_tx: Sender<String>) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    // let http_request: Vec<_> = buf_reader
    //     .lines()
    //     .map(|result| result.unwrap())
    //     .take_while(|line| !line.is_empty())
    //     .collect();

    println!("Request: {:#?}", request_line);

    let (status_line, contents_path) = match request_line.as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "src/hello.html"),
        "GET /shutdown HTTP/1.1" => {
            shutdown_tx.send("URI \"/shutdown\" recevied".to_string()).unwrap();
            ("HTTP/1.1 200 OK", "src/shutdown.html")
        }
        "GET /sleep HTTP/1.1" => {
            // some expensive job
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "src/sleep.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "src/404.html"),
    };

    let contents = read_to_string(contents_path).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
