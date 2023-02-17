#![allow(non_snake_case)]

use tempfile::tempdir;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tinydraw::ImageRGB8;
use clap::{command, value_parser, Arg, ArgAction, ArgMatches};


#[allow(clippy::needless_range_loop)]
fn hex_to_rgb(hex: &str) -> Result<[u8; 3], &'static str> {
    let mut rgb = [0; 3];
    let mut hex = hex.trim_start_matches('#');
    for i in 0..3 {
        match u8::from_str_radix(&hex[..2], 16) {
            Ok(v) => rgb[i] = v,
            Err(_) => return Err("Invalid hex color"),
        }
        hex = &hex[2..];
    }
    Ok(rgb)
}

fn main() {
    let bundled_ffmpeg = include_bytes!("../data/ffmpeg-compressed.exe");

    let arguments: ArgMatches = command!()
        .next_line_help(true)
        .arg(Arg::new("destination_file")
            .value_name("FILE")
            .help("The file to save the video to")
            .required(true)
            .value_parser(value_parser!(PathBuf)))
        .arg(Arg::new("video_length")
            .short('l')
            .long("length")
            .value_name("SECONDS")
            .help("The length of the video in seconds")
            .required(false)
            .value_parser(value_parser!(u128))
            .default_value("60"))
        .arg(Arg::new("fps")
            .short('f')
            .long("fps")
            .value_name("FPS")
            .help("The frames per second of the video")
            .required(false)
            .value_parser(value_parser!(u128))
            .default_value("60"))
        .arg(Arg::new("width")
            .short('w')
            .long("width")
            .value_name("WIDTH")
            .help("The width of the video")
            .required(false)
            .value_parser(value_parser!(u128))
            .default_value("1920"))
        .arg(Arg::new("height")
            .short('y')
            .long("height")
            .value_name("HEIGHT")
            .help("The height of the video")
            .required(false)
            .value_parser(value_parser!(u128))
            .default_value("1080"))
        .arg(Arg::new("num_of_balls")
            .short('n')
            .long("num_of_balls")
            .value_name("NUM")
            .help("The number of balls to simulate")
            .required(false)
            .value_parser(value_parser!(u128))
            .default_value("25"))
        .arg(Arg::new("background_color")
            .short('b')
            .long("background_color")
            .value_name("COLOR")
            .help("The background color of the video (HEX)")
            .required(false)
            .value_parser(hex_to_rgb)
            .default_value("#FFFFFF"))
        .arg(Arg::new("ball_color")
            .short('c')
            .long("ball_color")
            .value_name("COLOR")
            .help("The color of the balls (HEX)")
            .required(false)
            .value_parser(hex_to_rgb)
            .default_value("#000000"))
        .arg(Arg::new("ball_color_random")
            .short('C')
            .long("ball_color_random")
            .action(ArgAction::SetTrue)
            .help("Use random color for the balls")
            .required(false))
        .arg(Arg::new("ball_radius_min")
            .short('r')
            .long("ball_radius_min")
            .value_name("RADIUS")
            .help("The minimum radius of the balls")
            .required(false)
            .value_parser(value_parser!(u128))
            .default_value("50"))
        .arg(Arg::new("ball_radius_max")
            .short('R')
            .long("ball_radius_max")
            .value_name("RADIUS")
            .help("The maximum radius of the balls")
            .required(false)
            .value_parser(value_parser!(u128))
            .default_value("100"))
        .arg(Arg::new("ball_speed_min")
            .short('s')
            .long("ball_speed_min")
            .value_name("SPEED")
            .help("The minimum speed of the balls")
            .required(false)
            .value_parser(value_parser!(u128))
            .default_value("80"))
        .arg(Arg::new("ball_speed_max")
            .short('S')
            .long("ball_speed_max")
            .value_name("SPEED")
            .help("The maximum speed of the balls")
            .required(false)
            .value_parser(value_parser!(u128))
            .default_value("130"))
        .arg(Arg::new("ball_mass")
            .short('m')
            .long("ball_mass")
            .value_name("MASS")
            .help("The way of calculating the mass of the balls")
            .required(false)
            .value_parser(["circle", "ball"])
            .default_value("circle"))
        .get_matches();

    let dir = tempdir().unwrap();
    let ffmpeg_path = dir.path().join("ffmpeg.exe");
    let mut file = File::create(&ffmpeg_path).unwrap();
    file.write_all(bundled_ffmpeg).unwrap();
    drop(file);

    setup_simulation(arguments, ffmpeg_path.to_str().unwrap());
    drop(dir);
}

fn setup_simulation(cli_arguments: ArgMatches, ffmpeg_path: &str) {
    let destination_file = cli_arguments.get_one::<PathBuf>("destination_file").unwrap().to_str().unwrap();
    let video_length = *cli_arguments.get_one::<u128>("video_length").unwrap();
    let fps = *cli_arguments.get_one::<u128>("fps").unwrap();
    let width = *cli_arguments.get_one::<u128>("width").unwrap();
    let height = *cli_arguments.get_one::<u128>("height").unwrap();
    let num_of_balls = *cli_arguments.get_one::<u128>("num_of_balls").unwrap();
    let background_color = hex_to_rgb(cli_arguments.get_one::<&str>("background_color").unwrap()).unwrap();
    let ball_color = hex_to_rgb(cli_arguments.get_one::<&str>("ball_color").unwrap()).unwrap();
    let ball_color_random = cli_arguments.contains_id("ball_color_random");
    let ball_radius_min = *cli_arguments.get_one::<u128>("ball_radius_min").unwrap();
    let ball_radius_max = *cli_arguments.get_one::<u128>("ball_radius_max").unwrap();
    let ball_speed_min = *cli_arguments.get_one::<u128>("ball_speed_min").unwrap();
    let ball_speed_max = *cli_arguments.get_one::<u128>("ball_speed_max").unwrap();
    let ball_mass = *cli_arguments.get_one::<&str>("ball_mass").unwrap();

    let mut balls = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..num_of_balls {
        let radius = rng.gen_range(ball_radius_min..ball_radius_max);
        let speed = rng.gen_range(ball_speed_min..ball_speed_max);
        let angle = rng.gen_range(0.0..360.0);
        let x = rng.gen_range(radius..width - radius);
        let y = rng.gen_range(radius..height - radius);
        let color = if ball_color_random {
            let r = rng.gen_range(0..255);
            let g = rng.gen_range(0..255);
            let b = rng.gen_range(0..255);
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            ball_color.to_string()
        };
        let mass = match ball_mass {
            "circle" => radius * radius,
            "ball" => radius * radius * radius,
            _ => panic!("Invalid ball mass type"),
        };
        balls.push(Ball::new(x, y, radius, speed, angle, color, mass));
    }

    run_simulation(ffmpeg_path, destination_file, video_length, fps, width, height, background_color, balls);
}

fn run_simulation(ffmpeg_path: &str) {
    let mut ffmpeg_encoder = Command::new(ffmpeg_path)
        .arg("-y") // overwrite file if it already exists
        .arg("-f").arg("rawvideo") // interpret the information from stdin as "raw video"
        .arg("-pix_fmt").arg("rgb24") // every three bytes are [r, g, b] pixel
        .arg("-s").arg("1920x1080") // the size of the video
        .arg("-r").arg("60") // the fps of the video
        .arg("-an") // don't use audio
        .arg("-i").arg("-") // get data from stdin
        .arg("-c:v").arg("libx264") // encode to h264
        .arg("-crf").arg("0") // variable video bitrate
        .arg("test.mp4") // output file
        .stdin(Stdio::piped()).stderr(Stdio::piped()).stdout(Stdio::piped()) // stdin, stderr, and stdout are piped
        .spawn().unwrap(); // Run the child command

    let stdin = ffmpeg_encoder.stdin.as_mut().unwrap();

    let mut image: ImageRGB8 = ImageRGB8::new(1920, 1080, [255, 255, 255]);
    for _ in 0..10000 {
        image.clear();
        image.draw_circle(500, 500, 100, [0, 0, 0], 0, 1.0);
        stdin.write_all(image.to_bytes()).unwrap();
    }

    let output = ffmpeg_encoder.wait_with_output().unwrap();
    println!("{}", String::from_utf8(output.stdout).unwrap());
    println!("{}", String::from_utf8(output.stderr).unwrap());
}

struct Ball {
    x: u128,
    y: u128,
    m: u128,
    r: u128,
    v_x: u128,
    v_y: u128,
    color: [u8; 3]
}
