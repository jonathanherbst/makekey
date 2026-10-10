#![forbid(unsafe_code)]

use clap::Parser;
use rand::distr::SampleString;
use std::io::Write;

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
    Pass {
        /// Size of the password in characters
        #[arg(short, long, default_value_t = 16)]
        len: usize,
        /// Path of password pdf to generate
        path: Option<std::path::PathBuf>,
    },
    /// Generate a binary key
    Key {
        /// Size of the key in bytes
        #[arg(short, long, default_value_t = 32)]
        len: usize,
        /// Path of keyfile to generate
        path: Option<std::path::PathBuf>,
        /// Path of key pdf to generate (defaults to <path>.pdf)
        pdf_path: Option<std::path::PathBuf>,
    }
}

fn set_readonly(file: &mut std::fs::File) {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = file.metadata().expect("failed get file metadata").permissions();

    #[cfg(unix)]
    permissions.set_mode(0o0400);
    #[cfg(not(unix))]
    permissions.set_readonly(true);

    file.set_permissions(permissions).expect("failed to set file readonly");
}

fn key_command(len: usize, path: Option<std::path::PathBuf>, pdf_path: Option<std::path::PathBuf>) {
    let mut key: Vec<u8> = rand::random_iter().take(len).collect();
    if let Some(path) = path {
        if let Ok(mut key_file) = std::fs::File::create_new(path.clone()) {
            println!("generating key at {:?}", path);
            set_readonly(&mut key_file);
            key_file.write_all(&key).expect("failed to write key file");
        } else {
            println!("key file already exists, reading");
            key = std::fs::read(path.clone()).expect("failed to read key file");
            if key.len() != len {
                println!("requested len != len of keyfile, please remove the key file if you want to regenerate");
                return;
            }
        }

        let name: String = path.file_name().expect("unable to resolve file name").to_string_lossy().into_owned();
        let pdf_path = pdf_path.unwrap_or(path.clone().with_added_extension("pdf"));
        if let Ok(mut pdf_file) = std::fs::File::create(pdf_path.clone()) {
            println!("generating key pdf at {:?}", pdf_path);
            set_readonly(&mut pdf_file);
            makepdf(name.as_str(), hex::encode(&key).as_str(), pdf_file);
        } else {
            println!("failed to generate pdf");
        }
    }

    println!("{}", hex::encode(&key));
}

fn password_command(len: usize, pdf_path: Option<std::path::PathBuf>) {
    let mut rng = rand::rng();
    let pass = rand::distr::Alphanumeric.sample_string(&mut rng, len);
    if let Some(pdf_path) = pdf_path {
        if let Ok(mut pdf_file) = std::fs::File::create_new(pdf_path.clone()) {
            set_readonly(&mut pdf_file);
            let name: String = pdf_path.file_name().expect("unable to resolve file name").to_string_lossy().into_owned();
            makepdf(name.as_str(), pass.as_str(), pdf_file);
        } else {
            println!("pdf already exists, delete it and rerun if you want to generate a new password");
            return;
        }
    }
    println!("{}", pass);
}

pub fn makepdf(name: &str, keytxt: &str, w: impl Write) {
    static FONT: &[u8] = include_bytes!("../Inter/Inter_18pt-Black.ttf");

    let data = genpdf::fonts::FontData::new(FONT.to_vec(), None).expect("failed to load font");
    let font_family = genpdf::fonts::FontFamily {
        regular: data.clone(),
        bold: data.clone(),
        italic: data.clone(),
        bold_italic: data.clone(),
    };
    let mut doc = genpdf::Document::new(font_family);
    doc.set_title(name);
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    doc.push(genpdf::elements::Paragraph::new(name));
    doc.push(genpdf::elements::Paragraph::new(keytxt));

    let code = qrcode::QrCode::new(keytxt).expect("failed to create a qr code from the key");
    let image = code.render::<image::Luma<u8>>().
        min_dimensions(1000, 1000).
        build();
    let dyn_img = image::DynamicImage::ImageLuma8(image);
    let pdf_img = genpdf::elements::Image::from_dynamic_image(dyn_img).expect("failed to get image in pdf format");
    doc.push(pdf_img);

    doc.render(w).expect("failed to render the pdf");
}

fn main() {
    let args = Args::parse();

    match args.command {
        Some(Commands::Pass { len, path }) => password_command(len, path),
        Some(Commands::Key { len, path, pdf_path }) => key_command(len, path, pdf_path),
        None => password_command(16, None),
    }
}
