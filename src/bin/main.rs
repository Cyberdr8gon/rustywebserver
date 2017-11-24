extern crate rustywebserver;
extern crate regex;
use rustywebserver::ThreadPool;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::File;

use regex::Regex;
use std::fs;



fn main() {
    // refector this to make the error message show
    if ( !fs::metadata("index.html").unwrap().is_file() ||
            !fs::metadata("404.html").unwrap().is_file()) 
    {
        println!("Error: index.html or 404.html do not exist.");
        return
    }
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0;1024];
    stream.read(&mut buffer).unwrap();

    let buffer_str = String::from_utf8_lossy(&buffer[..]);

    println!("Request: {}", buffer_str);
    let get = b"GET / HTTP/1.1\r\n";

    
    let re = Regex::new(r"^GET /(?:([[:ascii:]]+))\sHTTP/[12].[0-9]+").unwrap();

    let caps = re.captures(&buffer_str);

    println!("Captures: {:?}", caps);

    let (status_line, filename) = match caps {
        Some(caps) => ("HTTP/1.1 200 OK\r\n\r\n", 
                        caps.get(1).map_or("", |m| m.as_str())),
        None if(buffer.starts_with(get)) => 
            ("HTTP/1.1 200 OK\r\n\r\n", "index.html"),
        None => ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    println!("Filename: {:?}", filename);

    let (status_line, filename) = match fs::metadata(filename) {
        Ok(metadata) => 
            if(metadata.is_file()) {
                (status_line, filename)
            } else {
                ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
            },
        _ => ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };


    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();

    file.read_to_string(&mut contents).unwrap();

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
