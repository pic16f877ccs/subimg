## subimg

A tool to hide sub-images in the image.

## Description

Writes pixels to a transparent area of an image from another image, leaving the alpha channel transparent. Supports PNG and TIFF types. The be hidded type can be PNG, TIFF, JPEG, GIF.

### Build

Build with Rust package manager.

```console
cargo b -r
```

### Usage:

#### Be hiddes image in image.

```console
subimg inputImage.png --be-hidden=subImage.jpeg --save=outputImage.png
```

#### Save invisible image.

```console
subimg imageInImage.png --save-invisible=invisibleImage.jpeg'
```

### Example:

#### Input image with alpha channel and jpeg type subimage.
|<img title="Image with alpha channel" src="md_img/gastropoda.png" alt="" width="325" height="">| <br> <img title="Additional image" src="md_img/picus.jpeg" alt="" width="325" height=""></br>|
|:-:|:-:|

#### Makes part of the image invisible.
```console
subimg gastropoda.png --be-hidden=picus.jpeg --save=gastropoda_picus.png
```
|<img title="Image in image" src="md_img/gastropoda_picus.png" alt="" width="325" height="">|
|:-:|

#### Save invisible image.
```console
subimg gastropoda_picus.png --save-invisible=picus.png
```
|<img title="Input image" src="md_img/gastropoda_picus.png" alt="image" width="325" height="">|<br> <img title="Otput subimage" src="md_img/picus.png" alt="image" width="325" height=""></br>|
|:-:|:-:|

## License

GNU General Public License v3.0
