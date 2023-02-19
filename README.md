# Circle-Bounce-rs

A CLI app to generate a video of bouncing circles 
(with continuous collision detection) written in Rust.

![screenshot](https://user-images.githubusercontent.com/40371578/219967105-4b5ff45c-2bdd-43aa-82a7-6e323da2b0f2.png)
Example: https://www.youtube.com/watch?v=ui5OTEb7xIs
```bash
circle-bounce-rs.exe video.mp4 -n 500 -r 10 -R 15 -C -l 3600
```

## Usage
1. from command line run `*.exe` file with one argument (destination file)
2. add other optional arguments
3. for list of other arguments run `*.exe --help`

```bash
>circle-bounce-rs.exe --help
A CLI app to generate a video of bouncing circles

Usage: circle-bounce-rs.exe [OPTIONS] <FILE>

Arguments:
  <FILE>
          The file to save the video to

Options:
  -l, --length <SECONDS>
          The length of the video in seconds [default: 60]
  -f, --fps <FPS>
          The frames per second of the video [default: 60]
  -w, --width <WIDTH>
          The width of the video [default: 1920]
  -y, --height <HEIGHT>
          The height of the video [default: 1080]
  -n, --num_of_balls <NUM>
          The number of balls to simulate [default: 25]
  -b, --background_color <COLOR>
          The background color of the video (HEX) [default: #ffffff]
  -c, --ball_color <COLOR>
          The color of the balls (HEX) [default: #000000]
  -C, --ball_color_random
          Use random color for the balls
  -r, --ball_radius_min <RADIUS>
          The minimum radius of the balls [default: 50]
  -R, --ball_radius_max <RADIUS>
          The maximum radius of the balls [default: 100]
  -s, --ball_speed_min <SPEED>
          The minimum speed of the balls [default: 80]
  -S, --ball_speed_max <SPEED>
          The maximum speed of the balls [default: 130]
  -m, --ball_mass <MASS>
          The way of calculating the mass of the balls [default: circle] [possible values: circle, ball]
  -h, --help
          Print help
  -V, --version
          Print version
```
