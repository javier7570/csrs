mod config;
mod servers;

fn main() {
   match config::read_config("config.ini") {
      Ok(config) => {
         servers::start_server(config).unwrap()
      },
      Err(error) => println!("{}", error)
   }
}
