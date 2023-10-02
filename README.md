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
<p align="center">
    <img title="Image with alpha channel" src="md_img/gastropoda.png" alt="" width="325" height="" hspae="10"> 
    <img title="Additional image" src="md_img/picus.jpeg" alt="" width="325" height="">
</p>

```console
subimg gastropoda.png --be-hidden=picus.jpeg --save=gastropoda_picus.png
```

#### Image in image ( sub image invisible ).

<img title="Image in image" src="md_img/gastropoda_picus.png" alt="" width="325" height="">

#### Save invisible image.

```console
subimg gastropoda_picus.png --save-invisible=picus.png
```
<p align="center">
    <img title="Input image" src="md_img/gastropoda_picus.png" alt="image" width="325" height="" hspace="10"> 
    <img title="Otput subimage" src="md_img/picus.png" alt="image" width="325" height="">
</p>

## License

GNU General Public License v3.0
