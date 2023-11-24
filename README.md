
[![Hits](https://hits.seeyoufarm.com/api/count/incr/badge.svg?url=https%3A%2F%2Fgithub.com%2Fpic16f877ccs%2Fsubimg&count_bg=%2379C83D&title_bg=%23555555&icon=&icon_color=%23E7E7E7&title=hits&edge_flat=false)](https://hits.seeyoufarm.com)

## subimg

A tool to hide another image in an image.

## Description

Writes pixels to a transparent area of an image from another image, leaving the alpha channel transparent. Supports PNG and TIFF types. The be hidded type can be PNG, TIFF, JPEG, GIF.

### Build

Build install with Rust package manager.

```console
cargo install subimg
```

### Usage:

#### Hides the image within the image.

```console
subimg inputImage.png --input=subImage.jpeg --output=outputImage.png
```

#### Saving an invisible image.

```console
subimg imageInImage.png --output-subimage=outputSubImage.jpeg
```

### Example:

#### Input image type PNG with transparent region and JPEG image file.
|<img title="Image with alpha channel" src="md_img/gastropoda.png" alt="" width="325" height="">| <br> <img title="Additional image" src="md_img/picus.jpeg" alt="" width="325" height=""></br>|
|:-:|:-:|

#### Makes the additional input image invisible.
```console
subimg gastropoda.png -i picus.jpeg -o gastropoda_picus.png
```
|<img title="Image in image" src="md_img/gastropoda_picus.png" alt="" width="325" height="">|
|:-:|

#### Extracts invisible image.
```console
subimg gastropoda_picus.png -O picus.jpeg
```
|<img title="Input image" src="md_img/gastropoda_picus.png" alt="image" width="325" height="">|<br> <img title="Otput subimage" src="md_img/picus.jpeg" alt="image" width="325" height=""></br>|
|:-:|:-:|

## License

GNU General Public License v3.0
