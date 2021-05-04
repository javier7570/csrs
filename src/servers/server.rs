use std::error::Error;
use std::collections::HashMap;

use std::net::{SocketAddr, Shutdown};
use std::io::{ErrorKind, Read, Write};
use std::sync::{Arc, RwLock};
   
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
   
use crate::config::Config;
   
const SERVER: Token = Token(0);

struct Node {
   addr: SocketAddr,
   stream: Option<TcpStream>
}
   
struct ServerData {
   addr: SocketAddr,
   nodes: Vec<Node>
}

fn start_server(server_data: &ServerData) -> Result<(), Box<dyn Error>> {
   let mut poll = Poll::new()?;
   let mut events = Events::with_capacity(128);
   let mut clients: HashMap<usize, TcpStream> = HashMap::new();
 
   let mut server = TcpListener::bind(server_data.addr)?;
   poll.registry().register(&mut server, SERVER, Interest::READABLE)?;

   let mut token_counter: usize = 0;

   loop {
      poll.poll(&mut events, None)?;

      for event in events.iter() {
         match event.token() {
            SERVER => {
               if event.is_error() {
                  match server.take_error() {
                     Ok(Some(e)) | Err(e) => return Err(Box::new(e)),
                     Ok(None) => panic!("Unknown error")
                  }
               }

               match server.accept() {
                  Ok((mut client, _)) => {
                     token_counter += 1;
                     poll.registry().register(&mut client, Token(token_counter), Interest::READABLE )?;
                     clients.insert(token_counter, client);
                  }
                  Err(ref e) if e.kind() == ErrorKind::WouldBlock =>
                     break,
                  Err(e) => 
                     panic!("Unexpected error: {}", e)
               }
            }

            Token(id) => {
               let client = clients.get(&id).unwrap();
               if event.is_error() {
                  match client.take_error() {
                     Ok(Some(e)) | Err(e) => return Err(Box::new(e)),
                     Ok(None) => panic!("Unknown error")
                  }
               }

               if event.is_writable() {
               }

               if event.is_readable() {
                  handle_rq(&mut client);
               }
               return Ok(());
            }

            // We don't expect any events with tokens other than those we provided.
             _ => unreachable!(),
         }
     }
   }

   Ok(())
}

fn send_message(mut stream: &TcpStream, message: &[u8]) -> bool {
   for s in &mut nodes {
      if s.stream.is_none() {
         if let Ok(stream) = TcpStream::connect(s.addr) {
            s.stream = Some(stream)
         }
         else {
            println!("{:#?}", s.addr);
            continue;
         }
      }
      let remote: &mut TcpStream = s.stream.as_mut().unwrap();
      remote.write(message).unwrap();
   }
   return true;
}

fn handle_rq(mut stream: &TcpStream) {
   let mut data = [0 as u8; 50]; // using 50 byte buffer
   while match stream.read(&mut data) {
      Ok(size) => {
         // echo everything!
         println!("Received message");
         send_message(&data[0..size]);         
         true
      },
      Err(_) => {
         println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
         stream.shutdown(Shutdown::Both).unwrap();
         false
      }
   } {}
}

pub fn create_server(config_lock: Arc<RwLock<Config>>) {
   let config = config_lock.read().unwrap();
   let nodes: Vec<Node> = config.servers.iter()
            .filter(|&(k, _)| *k != config.self_id)
            .map(|(_, v)| Node {addr: v.clone(), stream: None})
            .collect();

   let server = ServerData{ addr: SocketAddr::from(([0, 0, 0, 0], config.port)),
                            nodes: nodes };
   
   start_server(&server);  
}
