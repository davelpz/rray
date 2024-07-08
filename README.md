
<br/>
<div align="center">
<img src="https://raw.githubusercontent.com/davelpz/rray/master/project_icon.png" alt="Logo" width="80" height="80">
<h3 align="center">rray</h3>
<p align="center">
A simple raytracer
</p>
</div>

## About

![Product Screenshot](https://raw.githubusercontent.com/davelpz/rray/master/example1.png)

Rray is A raytracer implementation in Rust, following the book  "The Ray Tracer Challenge" by Jamis Buck.
### Built With

- [Rust](https://www.rust-lang.org/)

## Prerequisites

Before you begin, ensure you have met the following requirements:
* You have installed the latest version of Rust and Cargo.

## Getting Started

To build rray, follow these steps:

```bash
cargo build --release
```

The rray executable will be built in the target/release directory.

rray has these arguments:

```bash
> rray % ./target/release/rray -h                      
A simple raytracer

Usage: rray [OPTIONS] --scene <SCENE>

Options:
  -W, --width <WIDTH>    Width of the generated image, default is 800 [default: 800]
  -H, --height <HEIGHT>  Height of the generated image, default is 600 [default: 600]
  -s, --scene <SCENE>    Scene file in YAML format
  -o, --output <OUTPUT>  Name of the output file, default is output.png [default: output.png]
  -a, --aa <AA>          Anti-aliasing level (default 1) (max 5) [default: 1]
  -h, --help             Print help
  -V, --version          Print version

```
## Usage
Create your scene file in yaml format. Here is an example:
```yaml
camera:
  fov: 60
  from: [0, 2.5, -5.0]
  to: [0,1,0]
  up: [0,1,0]
lights:
  - type: point
    color: [1,1,1]
    position: [-10,10,-10]
scene:
  - type: plane
    transforms: []
    material:
      pattern:
        type: checker
        pattern_a:
          type: solid
          color: [0.25, 0.25, 0.75]
          transforms: []
        pattern_b:
          type: solid
          color: [0.75, 0.75, 0.75]
          transforms: []
        transforms:
          - type: scale
            amount: [1, 1, 1]
      ambient: 0.1
      diffuse: 0.9
      specular: 0
      shininess: 200
  - type: sphere
    transforms:
     - type: translate
       amount: [0, 1, 2]
     - type: scale
       amount: [0.5, 0.5, 0.5]
    material:
     pattern:
       type: solid
       color: [1, 0, 0]
       transforms: []
     ambient: 0.1
     diffuse: 0.9
     specular: 0.9
     shininess: 200
     reflective: 0.9
     transparency: 0.1
     refractive_index: 1.5
```

Then run the raytracer with the following command:

```bash
./target/release/rray -W 800 -H 400 -s <scene file> -o test.png
```

Get this image as output:
![Generated Image](https://raw.githubusercontent.com/davelpz/rray/master/examples/test1.png)


## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request
## License

Distributed under the MIT License. See [MIT License](https://opensource.org/licenses/MIT) for more information.
## Contact

David Lopez - [@davelpz](https://twitter.com/dlopez)

Project Link: [https://github.com/davelpz/rray](https://github.com/davelpz/rray)
