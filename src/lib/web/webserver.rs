// use std::{
//     io::{prelude::*, BufReader},
//     net::{TcpListener, TcpStream},
// };


// pub struct WebServer {
//     pub port: u16
// }

// impl WebServer {
//     pub fn run (&mut self) {
//         let listener = TcpListener::bind(format!("127.0.0.1:{:?}", self.port)).unwrap();

//         for stream in listener.incoming() {
//             let stream = stream.unwrap();

//             self.handle_connection(stream);

//             println!("Connection established!");
//         }
//     }

//     fn handle_connection (&mut self, mut stream: TcpStream) {
//         let buf_reader = BufReader::new(&mut stream);
//         let http_request: Vec<_> = buf_reader
//             .lines()
//             .map(|result| result.unwrap())
//             .take_while(|line| !line.is_empty())
//             .collect();
    
//         let response = "HTTP/1.1 OK\r\n\r\n";
        
//         stream.write_all(response.as_bytes()).unwrap();
    
//         println!("Request: {:#?}", http_request);
//     }

//     fn read_(&self, file_path: &str) -> io::Result<String> {
//         let file = File::open(file_path)?;
//         let reader = BufReader::new(file);
//         let mut content = String::new();

//         for line in reader.lines() {
//             content.push_str(&line?);
//             content.push('\n');
//         }

//         Ok(content)
//     }
// }