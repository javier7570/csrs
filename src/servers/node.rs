use std::net::{ SocketAddr, Shutdown, TcpListener, TcpStream };
//use std::collections::HashSet;
use std::sync::{ Arc, RwLock };
use std::io::{ Read, Write };
use crate::config::Config;

struct Node {
   addr: SocketAddr
   //clients: HashSet<SocketAddr>       
}

impl Node {
   fn start(&self) {
      let listener = TcpListener::bind(self.addr).unwrap();

      for stream in listener.incoming() {
         match stream {
            Ok(stream) => {
               println!("New connection");
               self.handle_rq(stream)
            },
            Err(e) => {
               println!("Error listening in Node thread: {}", e);
               /* connection failed */
            }
         }
      }
   }

   fn handle_rq(&self, mut stream: TcpStream) {
      let mut data = [0 as u8; 50]; // using 50 byte buffer
      while match stream.read(&mut data) {
         Ok(_) => {
            // echo everything!
            println!("Received message in NODE");
            stream.write(b"OK").unwrap();
            true            
         },
         Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
         }
      } {}
   }
}

pub fn create_node(config_lock: Arc<RwLock<Config>>) {
   let node = {
      let config = config_lock.read().unwrap();

      Node { addr: config.servers.get(&config.self_id).unwrap().clone() }
   };
    
   node.start();  
}