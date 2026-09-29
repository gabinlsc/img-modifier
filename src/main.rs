use clap::{Parser, ValueEnum};
use image::GenericImageView;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(ValueEnum, Clone, Debug)]
enum Filter {
    Bw,
    Invert,
}

#[derive(Parser, Debug)]
#[command(name = "img-modifier", about = "CLI ultra-rapide de retouche d'images")]
struct Cli {
    /// Fichier(s) image à traiter
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// Appliquer un filtre (bw, invert)
    #[arg(short, long, value_enum)]
    filter: Option<Filter>,

    /// Rogner en carré centré (1:1)
    #[arg(short, long)]
    square: bool,

    /// Dossier de sortie
    #[arg(short, long, default_value = "./output")]
    output_dir: PathBuf,
}

fn process_image(img_path: &Path, args: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut img = image::open(img_path)?;

    // 1. Crop centré en carré
    if args.square {
        let (w, h) = img.dimensions();
        let size = w.min(h);
        let x = (w - size) / 2;
        let y = (h - size) / 2;
        img = img.crop_imm(x, y, size, size);
    }

    // 2. Application du filtre
    if let Some(ref filter) = args.filter {
        match filter {
            Filter::Bw => {
                img = img.grayscale();
            }
            Filter::Invert => {
                img.invert();
            }
        }
    }

    // 3. Sauvegarde
    fs::create_dir_all(&args.output_dir)?;
    if let Some(file_name) = img_path.file_name() {
        let dest_path = args.output_dir.join(file_name);
        img.save(&dest_path)?;
        println!("Image traitée : {}", dest_path.display());
    }

    Ok(())
}

fn main() {
    let args = Cli::parse();

    for path in &args.inputs {
        if path.is_file() {
            if let Err(e) = process_image(path, &args) {
                eprintln!("Erreur sur {:?} : {}", path, e);
            }
        } else {
            eprintln!("Fichier introuvable : {:?}", path);
        }
    }
}