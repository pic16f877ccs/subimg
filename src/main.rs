use anyhow::anyhow;
use anyhow::Context;
use clap::{crate_name, crate_version, value_parser, Arg, ArgMatches, Command};
use image::{open, save_buffer, ColorType, ImageFormat};
use std::path::PathBuf;

const EXTRACT_ERR: &str = "Failed to extract integrated image";
type Size = (u32, u32);

struct ImgInImg {
    basic_img_data: Vec<u8>,
    basic_img_size: Size,
    other_img_data: Vec<u8>,
    other_img_size: Size,
}

impl ImgInImg {
    fn open_basic_image(app: &ArgMatches) -> anyhow::Result<ImgInImg> {
        let Some(path) = app.get_one::<PathBuf>("image") else {
            panic!()
        };
        let basic_img = open(path)
            .with_context(|| format!("Failed to read image file from {}", path.display()))?;
        if basic_img.color().has_alpha() {
            Ok(ImgInImg {
                basic_img_size: (basic_img.width(), basic_img.height()),
                basic_img_data: basic_img.into_rgba8().into_vec(),
                other_img_size: (0u32, 0u32),
                other_img_data: vec![],
            })
        } else {
            Err(anyhow!(
                "An image file without an alpha channel is not supported"
            ))
        }
    }

    fn open_other_image(&mut self, app: &ArgMatches) -> anyhow::Result<()> {
        if let Some(path) = app.get_one::<PathBuf>("input") {
            let other_img = open(path)
                .with_context(|| format!("Failed to read image file from {}", path.display()))?;
            self.other_img_size = (other_img.width(), other_img.height());
            self.other_img_data = other_img.into_rgb8().into_vec();
            self.merge_img_datas(app)?;
        }
        Ok(())
    }

    fn merge_img_datas(&mut self, app: &ArgMatches) -> anyhow::Result<()> {
        let size_as_bytes = self.size_to_bytes();
        let mut other_iter = self.other_img_data.iter();
        let basic_iter = self.basic_img_data.chunks_mut(4).filter_map(|chunk| {
            if chunk[3] == 0 {
                Some(&mut chunk[..3])
            } else {
                None
            }
        });

        if app.get_flag("fill") {
            for chunk in basic_iter {
                for basic_elem in chunk.iter_mut() {
                    if let Some(other_elem) = other_iter.next() {
                        *basic_elem = *other_elem;
                    } else {
                        other_iter = self.other_img_data.iter();
                    };
                }
            }
        } else {
            let mut size_other_iter = size_as_bytes.iter().chain(other_iter);
            'outer: for chunk in basic_iter {
                for basic_elem in chunk.iter_mut() {
                    let Some(size_other_elem) = size_other_iter.next() else {
                        break 'outer;
                    };
                    *basic_elem = *size_other_elem;
                }
            }

            if size_other_iter.next().is_some() {
                return Err(anyhow!("There is not enough free space in the image"));
            }
        }

        Ok(())
    }

    fn save_basic_image(&self, app: &ArgMatches) -> anyhow::Result<()> {
        if let Some(path) = app.get_one::<PathBuf>("output") {
            let format = ImageFormat::from_path(path)?;
            if app.get_flag("rgb") && (ImageFormat::Png == format || ImageFormat::Tiff == format) {
                save_buffer(
                    path,
                    &self.rgba_to_rgb(),
                    self.basic_img_size.0,
                    self.basic_img_size.1,
                    ColorType::Rgb8,
                )
                .with_context(|| {
                    format!(
                        "The image could not be written to the file {}",
                        path.display()
                    )
                })?;
                return Ok(());
            }

            save_buffer(
                path,
                &self.basic_img_data,
                self.basic_img_size.0,
                self.basic_img_size.1,
                ColorType::Rgba8,
            )
            .with_context(|| {
                format!(
                    "The image could not be written to the file {}",
                    path.display()
                )
            })?;
        }
        Ok(())
    }

    fn detach_img_data(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut detach_data_iter = self
            .basic_img_data
            .chunks(4)
            .filter_map(|chank| {
                if chank[3] == 0 {
                    Some(&chank[0..3])
                } else {
                    None
                }
            })
            .flatten()
            .cloned();

        self.other_img_size = (
            iter_to_size(&mut detach_data_iter)?,
            iter_to_size(&mut detach_data_iter)?,
        );
        let other_len = (self.other_img_size.0 * self.other_img_size.1) as usize * 3;
        let other_img_data = detach_data_iter.take(other_len).collect::<Vec<u8>>();
        if other_len != other_img_data.len() || other_img_data.len() < 12 {
            return Err(anyhow!(EXTRACT_ERR));
        }

        Ok(other_img_data)
    }

    fn save_other_image(&mut self, app: &ArgMatches) -> anyhow::Result<()> {
        if let Some(path) = app.get_one::<PathBuf>("output_subimg") {
            let color_type = ColorType::Rgb8;
            self.other_img_data = self.detach_img_data()?;

            save_buffer(
                path,
                &self.other_img_data,
                self.other_img_size.0,
                self.other_img_size.1,
                color_type,
            )?;
        }

        Ok(())
    }

    fn size_to_bytes(&self) -> [u8; 8] {
        let w = self.other_img_size.0.to_ne_bytes();
        let h = self.other_img_size.1.to_ne_bytes();
        [w[0], w[1], w[2], w[3], h[0], h[1], h[2], h[3]]
    }

    fn rgba_to_rgb(&self) -> Vec<u8> {
        self.basic_img_data
            .chunks(4)
            .flat_map(|chank| &chank[0..3])
            .cloned()
            .collect::<Vec<u8>>()
    }

    fn available_pixels(&self, app: &ArgMatches) {
        if app.get_flag("pixels") {
            let pixels = self
                .basic_img_data
                .chunks(4)
                .filter(|chunk| chunk[3] == 0)
                .count();

            println!("{} pixels available in the image", pixels);
        }
    }
}

fn iter_to_size(iter: &mut impl Iterator<Item = u8>) -> anyhow::Result<u32> {
    let Some(byte0) = iter.next() else {
        return Err(anyhow!(EXTRACT_ERR));
    };
    let Some(byte1) = iter.next() else {
        return Err(anyhow!(EXTRACT_ERR));
    };
    let Some(byte2) = iter.next() else {
        return Err(anyhow!(EXTRACT_ERR));
    };
    let Some(byte3) = iter.next() else {
        return Err(anyhow!(EXTRACT_ERR));
    };
    Ok(u32::from_ne_bytes([byte0, byte1, byte2, byte3]))
}

fn main() -> anyhow::Result<()> {
    let app = app_commands();
    let mut img_in_img = ImgInImg::open_basic_image(&app)?;
    img_in_img.open_other_image(&app)?;
    img_in_img.available_pixels(&app);
    img_in_img.save_other_image(&app)?;
    img_in_img.save_basic_image(&app)?;

    Ok(())
}

fn app_commands() -> ArgMatches {
    Command::new(crate_name!())
        .about("A tool to hide sub-images in the image")
        .after_help(
            "Example:
  subimg input.png -i subimage.png -o output.png",
        )
        .long_version(crate_version!())
        .args_override_self(true)
        .arg(
            Arg::new("image")
                .value_parser(value_parser!(PathBuf))
                .value_name("PATH")
                .help("Path to the input image file")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::new("fill")
                .short('f')
                .long("fill")
                .requires("rgb")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .help("Fill the transparent area with another image")
                .required(false),
        )
        .arg(
            Arg::new("pixels")
                .short('p')
                .long("pixels")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .help("Check the available pixels in the image")
                .required(false),
        )
        .arg(
            Arg::new("rgb")
                .short('r')
                .long("no-alpha")
                .action(clap::ArgAction::SetTrue)
                .num_args(0)
                .help("Save the image in PNG or TIFF format without an alpha channel")
                .required(false),
        )
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("PATH")
                .help("Path to sub image file")
                .value_parser(value_parser!(PathBuf))
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("output_subimg")
                .short('O')
                .conflicts_with_all(["output", "input"])
                .long("output-subimage")
                .value_name("PATH")
                .help("Output path for the sub-image file")
                .value_parser(value_parser!(PathBuf))
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("PATH")
                .help("Output path for the final image")
                .value_parser(value_parser!(PathBuf))
                .num_args(1)
                .required(false),
        )
        .get_matches()
}
