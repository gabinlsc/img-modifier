use clap::{Parser, ValueEnum};
use image::{imageops, DynamicImage, GenericImageView, ImageFormat};
use indicatif::{ProgressBar, ProgressStyle};
use printpdf::*;
use rayon::prelude::*;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(ValueEnum, Clone, Debug)]
enum Filter {
    Bw,
    Invert,
}

#[derive(Parser, Debug, Clone)]
#[command(
    name = "img-modifier",
    about = "Outil CLI ultra-rapide de manipulation et compilation d'images en PDF"
)]
struct Cli {
    /// Fichiers ou répertoires à traiter
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// Appliquer un filtre (bw, invert)
    #[arg(short, long, value_enum)]
    filter: Option<Filter>,

    /// Recadrer au format carré 1:1 centré
    #[arg(short, long)]
    square: bool,

    /// Ajuster la luminosité (-100 à 100)
    #[arg(long, default_value_t = 0)]
    brightness: i32,

    /// Dossier de destination pour les images exportées
    #[arg(short, long, default_value = "./output")]
    output_dir: PathBuf,

    /// Compiler toutes les images traitées dans un seul fichier PDF (ex: --to-pdf album.pdf)
    #[arg(long)]
    to_pdf: Option<PathBuf>,
}

fn collect_image_paths(inputs: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let valid_extensions = ["jpg", "jpeg", "png", "webp", "bmp"];

    for input in inputs {
        if input.is_file() {
            files.push(input.clone());
        } else if input.is_dir() {
            for entry in WalkDir::new(input).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        if valid_extensions.contains(&ext.to_lowercase().as_str()) {
                            files.push(path.to_path_buf());
                        }
                    }
                }
            }
        }
    }
    files
}

fn process_single_image(
    img_path: &Path,
    args: &Cli,
) -> Result<(DynamicImage, PathBuf), Box<dyn std::error::Error + Send + Sync>> {
    let mut img = image::open(img_path)?;

    // 1. Recadrage carré
    if args.square {
        let (w, h) = img.dimensions();
        let size = w.min(h);
        let x = (w - size) / 2;
        let y = (h - size) / 2;
        img = img.crop_imm(x, y, size, size);
    }

    // 2. Filtres
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

    // 3. Luminosité
    if args.brightness != 0 {
        img = img.brighten(args.brightness);
    }

    // 4. Écriture du fichier individuel
    fs::create_dir_all(&args.output_dir)?;
    let filename = img_path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("image.png");
    let dest_path = args.output_dir.join(filename);

    img.save(&dest_path)?;

    Ok((img, dest_path))
}

fn create_pdf_from_images(
    images: &[PathBuf],
    pdf_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if images.is_empty() {
        return Ok(());
    }

    // Format A4 portrait : 210 x 297 mm
    let page_width = Mm(210.0);
    let page_height = Mm(297.0);

    let (doc, page1, layer1) =
        PdfDocument::new("Images Exportées", page_width, page_height, "Layer 1");

    for (i, img_file) in images.iter().enumerate() {
        let (current_page, current_layer) = if i == 0 {
            (page1, layer1)
        } else {
            let (p, l) = doc.add_page(page_width, page_height, format!("Page {}", i + 1));
            (p, l)
        };

        let current_layer = doc.get_page(current_page).get_layer(current_layer);

        let mut image_file = File::open(img_file)?;
        let image = Image::try_from(image::codecs::jpeg::JpegDecoder::new(&mut image_file)
            .map_err(|_| "Convertissez vos images en JPG pour le PDF")?)?;

        // Placer et adapter l'image au centre de la page A4
        image.add_to_layer(
            current_layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(10.0)),
                translate_y: Some(Mm(10.0)),
                scale_x: Some(0.8),
                scale_y: Some(0.8),
                ..Default::default()
            },
        );
    }

    let file = File::create(pdf_path)?;
    doc.save(&mut BufWriter::new(file))?;
    Ok(())
}

fn main() {
    let args = Cli::parse();
    let file_list = collect_image_paths(&args.inputs);

    if file_list.is_empty() {
        eprintln!(" Aucune image compatible trouvée dans les chemins fournis.");
        return;
    }

    println!("{} fichier(s) détecté(s). Démarrage...", file_list.len());

    let pb = ProgressBar::new(file_list.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Traitement parallèle avec Rayon
    let processed_results: Vec<Result<(DynamicImage, PathBuf), _>> = file_list
        .par_iter()
        .map(|path| {
            let res = process_single_image(path, &args);
            pb.inc(1);
            res
        })
        .collect();

    pb.finish_with_message("Traitement terminé !");

    let mut successful_paths = Vec::new();
    for res in processed_results {
        match res {
            Ok((_, path)) => successful_paths.push(path),
            Err(e) => eprintln!("Échec : {}", e),
        }
    }

    println!(
        "\n{} images traitées avec succès dans {:?}",
        successful_paths.len(),
        args.output_dir
    );

    // Export PDF si demandé
    if let Some(ref pdf_target) = args.to_pdf {
        println!("Génération du PDF : {:?}...", pdf_target);
        if let Err(e) = create_pdf_from_images(&successful_paths, pdf_target) {
            eprintln!("Erreur génération PDF : {}", e);
        } else {
            println!("PDF généré avec succès !");
        }
    }
}