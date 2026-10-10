#![forbid(unsafe_code)]

use clap::Parser;
use rand::distr::SampleString;
use std::io::Write;

mod keypdf;

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: Option<std::path::PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    /// Generate an alphanum password
    Password {
        /// Size of the password in characters
        #[arg(default_value_t = 16)]
        len: usize,
    },
    /// Generate a binary key
    Key {
        /// Size of the key in bytes
        #[arg(default_value_t = 32)]
        len: usize,
    }
}

fn key_command(len: usize, path: Option<std::path::PathBuf>) {
    let key: Vec<u8> = rand::random_iter().take(len).collect();
    if let Some(path) = path {
        let name: String = path.file_name().expect("unable to resolve file name").to_string_lossy().into_owned();
        let pdf_path = path.clone().with_added_extension("pdf");
        let mut key_file = std::fs::File::create_new(path.clone()).expect("failed to create key file");
        let pdf_file = std::fs::File::create_new(pdf_path).expect("failed to create pdf file");
        key_file.write_all(&key).expect("failed to write key file");
        keypdf::makepdf(name.as_str(), hex::encode(&key).as_str(), pdf_file);
        println!("successfully generated key at {:?}", path);
    }

    println!("{}", hex::encode(&key));
}

fn password_command(len: usize) {
    let mut rng = rand::rng();
    let pass = rand::distr::Alphanumeric.sample_string(&mut rng, len);
    println!("{}", pass);
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Commands::Password { len }) => password_command(len),
        Some(Commands::Key { len }) => key_command(len, args.path),
        None => password_command(16),
    }
}
