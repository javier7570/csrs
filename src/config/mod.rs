use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{ BufRead, BufReader };
use std::net::SocketAddr;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
   pub port: u16,
   pub self_id: i32,
   pub servers: HashMap<i32, SocketAddr>
}

#[derive(Debug)]
pub enum ConfigError {
   Io(String, String),
   Parse(usize, String, String),
   Missing(String, String)
}

impl fmt::Display for ConfigError {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match &*self {
         ConfigError::Io(file, message) => write!(f, "Error reading {}: {}", file, message),
         ConfigError::Parse(line, file, message) => write!(f, "Error reading file {} in line {}: {}", file, line, message),
         ConfigError::Missing(file, message) =>  write!(f, "Error reading {}: {}", file, message)
      }
   }
}

impl Error for ConfigError {
   fn description(&self) -> &str {
      match *self {
         _ => "None"
      }
   }
}

pub fn read_config(config_file: &str) -> Result<Config, ConfigError> {
    
   let file = File::open(config_file).map_err(|_| ConfigError::Io(config_file.to_string(), "Unable to open config file".to_string()))?;

   let reader = BufReader::new(file);
   let mut port: Option<u16> = None;
   let mut self_id: Option<i32> = None;
   let mut server_addrs: HashMap<i32, SocketAddr> = HashMap::new();

   for (i, res) in reader.lines().enumerate() {
      let line = res.map_err(|err| ConfigError::Parse(i + 1, config_file.to_string(), err.to_string()))?;
      let vec: Vec<&str> = line.split("=").collect();
      if vec.len() != 2 {
         return Err(ConfigError::Parse(i + 1, config_file.to_string(), "Invalid line format".to_string()))
      }

      let attr = vec[0].trim();
      let value = vec[1].trim();

      if attr.starts_with("#") {
         continue;
      }
      if attr == "self" {
         self_id = Some(value.parse::<i32>().map_err(|_| ConfigError::Parse(i + 1, config_file.to_string(), "Error parsing self identifier".to_string()))?);
      }
      else if attr == "port" {
         port = Some(value.parse::<u16>().map_err(|_| ConfigError::Parse(i + 1, config_file.to_string(), "Error parsing port value".to_string()))?);
      }
      else {
         let id = attr.parse::<i32>().map_err(|_| ConfigError::Parse(i + 1, config_file.to_string(), "Error parsing server id".to_string()))?;
         let ip_addr = value.parse::<SocketAddr>()
               .map_err(|_| ConfigError::Parse(i + 1, config_file.to_string(), "Error parsing IP address".to_string()))?;
         server_addrs.insert(id, ip_addr);
      }
   }

   let self_val: i32 = match self_id {
      Some(val) => val,
      None => return Err(ConfigError::Missing(config_file.to_string(), "Self identifier not specified".to_string()))
   };

   let port_val: u16 = match port {
      Some(val) => val,
      None => return Err(ConfigError::Missing(config_file.to_string(), "Port not specified".to_string()))
   };

   if !server_addrs.contains_key(&self_val) {
      return Err(ConfigError::Missing(config_file.to_string(), "Self server address not specified".to_string()))
   }
    
   Ok(Config{port: port_val, self_id: self_val, servers: server_addrs})
}
