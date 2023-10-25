use cev::Cev;
use clap::{crate_version, value_parser, Arg, ArgMatches, Command};
use image::{open, save_buffer, ColorType, DynamicImage, ImageFormat, ImageResult};
use std::error;
use std::path::PathBuf;

type Size = (u32, u32);
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Default)]
struct ImgInImg {
    img_data: Vec<u8>,
    img_size: Size,
    img_alpha: Option<ColorType>,
    sub_img_data: Option<Vec<u8>>,
    sub_img_size: Size,
    sub_img_len: Option<usize>,
}

impl ImgInImg {
    fn new() -> ImgInImg {
        ImgInImg::default()
    }

    fn open_image(&mut self, app: &ArgMatches) -> ImageResult<()> {
        let path = app
            .get_one::<PathBuf>("image")
            .map(|path| path.to_path_buf())
            .expect("internal error opening file");
        let img = open(path)?;
        self.img_alpha = match img.color() {
            ColorType::Rgba8 => Some(ColorType::Rgba8),
            _ => None,
        };
        self.img_size = (img.width(), img.height());
        self.img_data = img.into_rgba8().into_vec();
        self.sub_img_size = self.sub_img_size();
        self.sub_img_len = self.sub_img_len();
        self.sub_img_data = self.sub_img_data();
        Ok(())
    }

    fn sub_img_size(&self) -> Size {
        let mut encoded_size = Vec::from(&self.img_data[..10]);
        encoded_size.remove(9);
        encoded_size.remove(4);
        let mut width_buf = [0, 0, 0, 0];
        let mut height_buf = [0, 0, 0, 0];
        width_buf.clone_from_slice(&encoded_size[..4]);
        height_buf.clone_from_slice(&encoded_size[4..8]);
        (
            u32::from_ne_bytes(width_buf),
            u32::from_ne_bytes(height_buf),
        )
    }

    fn sub_img_len(&self) -> Option<usize> {
        let Some(sub_img_len) = self.sub_img_size.0.checked_mul(self.sub_img_size.1) else {
            return None;
        };
        let Some(sub_img_len) = sub_img_len.checked_mul(4) else {
            return None;
        };
        let Some(sub_img_len) = sub_img_len.checked_add(12) else {
            return None;
        };
        if sub_img_len <= 12 {
            return None;
        };
        Some(sub_img_len as usize)
    }

    fn sub_img_data(&mut self) -> Option<Vec<u8>> {
        let Some(sub_img_len) = self.sub_img_len else {
            return None;
        };

        let sub_img_data = self
            .img_data
            .chunks_mut(4)
            .filter(|chunk| chunk[3] == 0)
            .take(sub_img_len / 4)
            .flat_map(|chunk| {
                if chunk[0..3] == [0, 0, 0] {
                    [255, 255, 255, 255]
                } else {
                    chunk[3] = 255;
                    [chunk[0], chunk[1], chunk[2], chunk[3]]
                }
            })
            .collect::<Vec<_>>();

        if sub_img_data.len() != sub_img_len {
            return None;
        }
        Some(sub_img_data)
    }

    fn subvec_to_vec(&mut self, sub_vec: Vec<u8>) -> Result<()> {
        let sub_vec_len = sub_vec.len() / 4;
        let mut sub_iter = sub_vec.iter();
        let iter = self
            .img_data
            .chunks_mut(4)
            .filter(|chunk| chunk[3] == 0)
            .take(sub_vec_len);

        for chunk in iter {
            for elem in chunk.iter_mut() {
                let Some(sub_elem) = sub_iter.next() else {
                    return Err("the inner error of writing a sub-vector into a vector".into());
                };
                *elem = *sub_elem;
            }
        }

        if sub_iter.next().is_some() {
            return Err("there is not enough free space in the image".into());
        };
        Ok(())
    }

    fn available_pixels(&self, app: &ArgMatches) {
        if app.get_flag("pixels") {
            if self.img_alpha.is_some() {
                let pixels = self
                    .img_data
                    .chunks(4)
                    .filter(|chunk| chunk[3] == 0)
                    .count()
                    / 1_000_000;

                println!("\n {} megapixels available in the image", pixels);
            } else {
                println!("\n there are no available pixels in the image");
            }
        }
    }

    fn add_sub_img_data(&mut self, path: &PathBuf) -> Result<()> {
        let sub_img = open_sub_img(path)?;
        let sub_img_size = encode_sub_img_size(&sub_img);
        let sub_img_data = sub_img.into_rgba8().into_vec();
        let mut tmp_data = Cev::from_vec(sub_img_data);
        tmp_data.append(&mut Cev::from(sub_img_size));
        let mut sub_img_data = tmp_data.into_vec();
        img_to_invisible(&mut sub_img_data);
        self.subvec_to_vec(sub_img_data)?;
        Ok(())
    }

    fn all_alpha_max(&mut self, app: &ArgMatches) {
        if app.get_flag("all") {
            self.img_data
                .iter_mut()
                .skip(3)
                .step_by(4)
                .for_each(|alpha| {
                    *alpha = 255;
                });
        }
    }

    fn save_img_in_img(&mut self, app: &ArgMatches) -> Result<()> {
        if let Some(path) = app
            .get_one::<PathBuf>("input")
            .map(|path| path.to_path_buf())
        {
            let Some(_) = self.img_alpha else {
                return Err("Unsupported image input format. Try `tiff` or `png` format.".into());
            };
            self.add_sub_img_data(&path)?;
        }
        Ok(())
    }

    fn save_sub_img(&self, app: &ArgMatches) -> Result<()> {
        if let Some(path) = app.get_one::<PathBuf>("output_subimg") {
            let Some(ref img) = self.sub_img_data else {
                return Err("subimage decoding error".into());
            };
            let color_type = ColorType::Rgba8;
            let size = self.sub_img_size;
            save_buffer(path, &img[12..], size.0, size.1, color_type)?;
        }
        Ok(())
    }

    fn save_img(&self, app: &ArgMatches) -> Result<()> {
        if let Some(path) = app.get_one::<PathBuf>("output") {
            let color_type = ColorType::Rgba8;
            let format = ImageFormat::from_path(path)?;
            if app.contains_id("input") {
                let (ImageFormat::Png | ImageFormat::Tiff) = format else {
                    return Err(
                        "Unsupported image output format. Try `tiff` or `png` format.".into(),
                    );
                };
            }
            save_buffer(
                path,
                &self.img_data,
                self.img_size.0,
                self.img_size.1,
                color_type,
            )?;
        }
        Ok(())
    }
}

fn open_sub_img(path: &PathBuf) -> ImageResult<DynamicImage> {
    open(path)
}

fn encode_sub_img_size(sub_img: &DynamicImage) -> Vec<u8> {
    let mut size = Vec::from(sub_img.width().to_ne_bytes());
    size.insert(3, 0);
    size.append(&mut Vec::from(sub_img.height().to_ne_bytes()));
    size.insert(7, 0);
    size.append(&mut vec![0, 0]);
    size
}

fn img_to_invisible(img_data: &mut [u8]) {
    img_data.iter_mut().skip(3).step_by(4).for_each(|alpha| {
        *alpha = 0;
    });
}

fn main() -> Result<()> {
    let app = app_commands();
    let mut img_in_img = ImgInImg::new();
    img_in_img.open_image(&app)?;
    img_in_img.available_pixels(&app);
    img_in_img.save_sub_img(&app)?;
    img_in_img.save_img_in_img(&app)?;
    img_in_img.all_alpha_max(&app);
    img_in_img.save_img(&app)?;
    Ok(())
}

fn app_commands() -> ArgMatches {
    Command::new("subimg")
        .about("A tool to hide sub-images in the image")
        .long_version(crate_version!())
        .author("    by PIC16F877ccs")
        .args_override_self(true)
        .arg(
            Arg::new("image")
                .value_parser(value_parser!(PathBuf))
                .value_name("PAPH")
                .help("Path to input image file")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::new("pixels")
                .short('p')
                .long("pixels")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .help("Available pixels in image")
                .required(false),
        )
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("PAPH")
                .help("Path to sub image file")
                .value_parser(value_parser!(PathBuf))
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("all")
                .short('a')
                .long("all")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .help("All pixels are visible")
                .required(false),
        )
        .arg(
            Arg::new("output_subimg")
                .short('O')
                .conflicts_with_all(["output", "input"])
                .long("output-subimage")
                .value_name("PAPH")
                .help("Output sub image file")
                .value_parser(value_parser!(PathBuf))
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("PAPH")
                .help("Output image file")
                .value_parser(value_parser!(PathBuf))
                .num_args(1)
                .required(false),
        )
        .get_matches()
}
