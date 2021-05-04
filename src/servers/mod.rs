use std::error::Error;
use std::thread;
use std::sync::{ Arc, RwLock};
use crate::config::Config;

mod server;
mod node;

pub fn start_server(config: Config) -> Result<(), Box<dyn Error>> {
   let config_lock = Arc::new(RwLock::new(config));
   /*let port = config.port;
   let servers: Vec<SocketAddr> = config.servers.iter()
               .filter(|&(k, _)| *k != config.self_id)
               .map(|(_, v)| v.clone())
               .collect();

   println!("{:#?}", servers);*/
   let config_server = Arc::clone(&config_lock);
   let server = thread::spawn(move || {
      server::create_server(config_server)
   });

   let config_node = Arc::clone(&config_lock);
   let node = thread::spawn(move || {
      node::create_node(config_node)
   });

   server.join().unwrap();
   node.join().unwrap();
   Ok(())
}
