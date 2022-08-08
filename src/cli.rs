use clap::Parser;
use std::path::PathBuf;

/// A Block Game Server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
   /// Adress to listen on
   #[clap(short, long, value_parser, default_value_t = String::from("0.0.0.0:25565"))]
   pub adress: String,

   /// x size
   #[clap(short, long, value_parser = clap::value_parser!(i16).range(1..), default_value_t = 256)]
   pub x_size: i16,

   /// y size
   #[clap(short, long, value_parser = clap::value_parser!(i16).range(1..), default_value_t = 32)]
   pub y_size: i16,

   /// x size
   #[clap(short, long, value_parser = clap::value_parser!(i16).range(1..), default_value_t = 256)]
   pub z_size: i16,

   /// Ground height
   #[clap(short, long, value_parser, default_value_t = 20.0)]
   pub height: f64,

   /// Data directory
   #[clap(short, long, value_parser, default_value_os_t = PathBuf::from("data"))]
   pub data: PathBuf,

   /// Server name shown when connecting
   #[clap(short, long, value_parser, default_value_t = String::from("Classicl Server"))]
   pub name: String,

   /// Server MOTD shown when connecting
   #[clap(short, long, value_parser, default_value_t = String::from("hosted with style"))]
   pub motd: String,
}