// icnsutil documentation: https://developer.apple.com/library/archive/documentation/GraphicsAnimation/Conceptual/HighResolutionOSX/Optimizing/Optimizing.html

use std::fs::create_dir_all;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use clap::Parser;
use const_format::formatcp;

use crate::utils::construct_entry_name;

mod utils;

const GIT_HASH: &str = env!("GIT_HASH");
const GIT_BRANCH: &str = env!("GIT_BRANCH");
const GIT_VERSION: &str = env!("GIT_VERSION");
const BUILD_DATE: &str = env!("BUILD_DATE");

const CLAP_VERSION: &str = formatcp!("{GIT_VERSION} [{GIT_BRANCH}, {GIT_HASH}, {BUILD_DATE}]");

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum ConvertMode {
	#[default] Auto,
	ICO,
	PNG,
}

#[derive(Parser, Debug)]
#[command(version = CLAP_VERSION, about = "Derives an image with alpha channel from two alpha-less images")]
struct Args {
	#[clap(short, long, value_enum, help = "The conversion target", default_value_t = ConvertMode::default())]
	convert: ConvertMode,
	#[clap(short, long, help = "The output file/folder")]
	output: Option<PathBuf>,
	#[clap(short='O', long, help = "Overwrite the output file/folder")]
	overwrite: bool,

	#[clap(help = "The input file/folder")]
	input: PathBuf,
}

// supported icon sizes
// source: https://learn.microsoft.com/en-us/windows/apps/design/style/iconography/app-icon-construction#icon-scaling
static ICON_SIZES: &[u32] = &[16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 256];

fn create_ico<P: AsRef<Path>>(input_dir: P, output_file: P) -> Result<(), String> {
	let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

	for icon_size in ICON_SIZES {
		let size_png = format!("{icon_size}.png");
		let png_file = match std::fs::File::open(input_dir.as_ref().join(&size_png)) {
			Ok(result) => result,
			Err(error) => match error.kind() {
				ErrorKind::NotFound => {
					continue // ignore this one
				},
				_ => {
					println!("Can't open {size_png}: {error}");
					continue
				}
			},
		};

		let image = ico::IconImage::read_png(png_file).unwrap();
		if (image.width(), image.height()) != (*icon_size, *icon_size) {
			println!("{size_png} must be {icon_size}×{icon_size} px, but is {}×{} px instead.", image.width(), image.height());
			continue
		}

		icon_dir.add_entry(ico::IconDirEntry::encode_as_png(&image).unwrap());
		println!("Added {size_png}");
	}

	if icon_dir.entries().is_empty() {
		return Err("No suitable PNG files found.".parse().unwrap());
	}

	let ico_file = std::fs::File::create(&output_file).unwrap();
	icon_dir.write(ico_file).unwrap();

	println!("Created {} with {} images.", output_file.as_ref().file_name().unwrap().to_str().unwrap(), icon_dir.entries().len());

	Ok(())
}

fn extract_pngs<P: AsRef<Path>>(input_file: P, output_dir: P) -> Result<(), String> {
	let file = std::fs::File::open(input_file).expect("Can't open input .ico file");
	let icon_dir = ico::IconDir::read(file).expect("Input file is invalid");

	create_dir_all(&output_dir).expect("Can't create output folder");

	for entry in icon_dir.entries() {
		let image = entry.decode().expect("Can't decode image");

		let file_name = construct_entry_name(entry);
		let png_file = match std::fs::File::create(output_dir.as_ref().join(&file_name)) {
			Ok(result) => result,
			Err(error) => match error.kind() {
				_ => {
					println!("Can't create {file_name}: {error}");
					continue
				}
			},
		};
		image.write_png(png_file).expect("Can't write PNG file");
		println!("Extracted {file_name}");
	}

	Ok(())
}

fn main() -> Result<(), String> {
	let args = Args::parse();

	// make sure args.output is usable
	let mut temp_buf = PathBuf::from(args.input.parent().expect("Can't get input file parent"));
	let file_stem = args.input.file_stem().expect("Can't get file stem").to_str().unwrap();
	match args.convert {
		ConvertMode::Auto => {
			if args.input.is_file() && args.input.extension().unwrap_or_default().eq_ignore_ascii_case("ico") {
				// input is .ico file
				temp_buf.push(file_stem);
			} else if args.input.is_dir() {
				// input is folder
				temp_buf.push(format!("{file_stem}.ico"));
			}
		}
		ConvertMode::ICO => { temp_buf.push(format!("{file_stem}.ico")); }
		ConvertMode::PNG => { temp_buf.push(file_stem); }
	}

	let output_path = args.output.unwrap_or(temp_buf);
	println!("output: {}", output_path.display());

	if output_path.exists() && !args.overwrite {
		if output_path.is_file() {
			return Err("The output file already exists. Specify --overwrite to overwrite it.".parse().unwrap());
		} else if output_path.is_dir() {
			return Err("The output folder already exists. Specify --overwrite to overwrite it.".parse().unwrap());
		}
	}

	match args.convert {
		ConvertMode::Auto => {
			if args.input.is_file() && args.input.extension().unwrap_or_default().eq_ignore_ascii_case("ico") {
				// input is .ico file
				extract_pngs(args.input, output_path).unwrap()
			} else if args.input.is_dir() {
				// input is folder
				create_ico(args.input, output_path).unwrap()
			}
		}
		ConvertMode::ICO => { create_ico(args.input, output_path).unwrap() }
		ConvertMode::PNG => { extract_pngs(args.input, output_path).unwrap() }
	}

	Ok(())
}
