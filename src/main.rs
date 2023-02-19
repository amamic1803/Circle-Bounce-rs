#![allow(non_snake_case)]

use std::f64::consts::PI;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use clap::{Arg, ArgAction, ArgMatches, command, value_parser};
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use tempfile::tempdir;
use tinydraw::ImageRGB8;


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
            .default_value("#ffffff"))
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
    if video_length == 0 {
        println!("Video length must be greater than 0");
        return;
    }
    let fps = *cli_arguments.get_one::<u128>("fps").unwrap();
    if fps == 0 {
        println!("FPS must be greater than 0");
        return;
    }
    let width = *cli_arguments.get_one::<u128>("width").unwrap();
    if width == 0 {
        println!("Width must be greater than 0");
        return;
    }
    let height = *cli_arguments.get_one::<u128>("height").unwrap();
    if height == 0 {
        println!("Height must be greater than 0");
        return;
    }
    let num_of_balls = *cli_arguments.get_one::<u128>("num_of_balls").unwrap();
    if num_of_balls == 0 {
        println!("Number of balls must be greater than 0");
        return;
    }
    let background_color: [u8; 3] = *cli_arguments.get_one::<[u8; 3]>("background_color").unwrap();
    let ball_color: [u8; 3] = *cli_arguments.get_one::<[u8; 3]>("ball_color").unwrap();
    let ball_color_random: bool = cli_arguments.get_flag("ball_color_random");
    let ball_radius_min: f64 = *cli_arguments.get_one::<u128>("ball_radius_min").unwrap() as f64;
    let ball_radius_max: f64 = *cli_arguments.get_one::<u128>("ball_radius_max").unwrap() as f64;
    if ball_radius_max < ball_radius_min {
        println!("Maximum radius must be greater than or equal to minimum radius");
        return;
    } else if ball_radius_max == 0.0 {
        println!("Maximum radius must be greater than 0");
        return;
    }
    let ball_speed_min: f64 = *cli_arguments.get_one::<u128>("ball_speed_min").unwrap() as f64;
    let ball_speed_max: f64 = *cli_arguments.get_one::<u128>("ball_speed_max").unwrap() as f64;
    if ball_speed_max < ball_speed_min {
        println!("Maximum speed must be greater than or equal to minimum speed");
        return;
    } else if ball_speed_max == 0.0 {
        println!("Maximum speed must be greater than 0");
        return;
    }
    let ball_mass = cli_arguments.get_one::<String>("ball_mass").unwrap().as_str();

    let mut balls: Vec<Ball> = vec![];
    let mut rng: ThreadRng = thread_rng();
    for _ in 0..num_of_balls {
        // radius
        let radius = rng.gen_range((ball_radius_min as u128)..=(ball_radius_max as u128)) as f64;

        // x, y
        let mut tries = 0;
        let mut x: f64;
        let mut y: f64;
        'outer: loop {
            tries += 1;
            if tries > 10_000 {
                println!("Can't fit all balls in the given area");
                return;
            }
            x = rng.gen_range(radius..(width as f64 - radius - 1.0));
            y = rng.gen_range(radius..(height as f64 - radius - 1.0));

            for ball in &balls {
                if ((x - ball.x).abs() <= (radius + ball.r)) && ((y - ball.y).abs() <= (radius + ball.r)) {
                    continue 'outer;
                }
            }
            break;
        }

        // mass
        let mass = match ball_mass {
            "circle" => radius * radius * PI,
            "ball" => (radius * radius * radius * PI * 4.0) / 3.0,
            _ => panic!("Invalid ball mass type"),
        };

        // speed (vx, vy)
        let speed_x: f64 = (*[-1, 1].choose(&mut rng).unwrap() as f64) * rng.gen_range(ball_speed_min..=ball_speed_max);
        let speed_y: f64 = (*[-1, 1].choose(&mut rng).unwrap() as f64) * rng.gen_range(ball_speed_min..=ball_speed_max);

        // color
        let color: [u8; 3] = if ball_color_random {
            let mut color_temp: [u8; 3] = [rng.gen_range(0..=255), rng.gen_range(0..=255), rng.gen_range(0..=255)];
            while color_temp == background_color {
                color_temp = [rng.gen_range(0..=255), rng.gen_range(0..=255), rng.gen_range(0..=255)];
            }
            color_temp
        } else {
            ball_color
        };

        balls.push(Ball::new(x, y, mass, radius, speed_x, speed_y, color));
    }

    drop(rng);
    run_simulation(ffmpeg_path, destination_file, video_length, fps, width, height, background_color, balls);
}

#[allow(clippy::too_many_arguments)]
fn run_simulation(ffmpeg_path: &str, destination_file: &str, video_length: u128, fps: u128, width: u128, height: u128, background_color: [u8; 3], mut balls: Vec<Ball>) {
    let mut ffmpeg_encoder = Command::new(ffmpeg_path)
        .arg("-y") // overwrite file if it already exists
        .arg("-f").arg("rawvideo") // interpret the information from stdin as "raw video"
        .arg("-pix_fmt").arg("rgb24") // every three bytes are [r, g, b] pixel
        .arg("-s").arg(format!("{}x{}", width, height)) // the size of the video
        .arg("-r").arg(fps.to_string()) // the fps of the video
        .arg("-an") // don't use audio
        .arg("-i").arg("-") // get data from stdin
        .arg("-c:v").arg("libx264") // encode to h264
        .arg("-crf").arg("0") // variable video bitrate
        .arg(destination_file) // output file
        .stdin(Stdio::piped()).stderr(Stdio::piped()).stdout(Stdio::piped()) // stdin, stderr, and stdout are piped
        .spawn().unwrap(); // Run the child command
    let stdin = ffmpeg_encoder.stdin.as_mut().unwrap();

    let width = width as f64;
    let height = height as f64;
    let interval = 1.0 / (fps as f64);

    let mut times: Vec<Vec<Option<f64>>> = vec![];
    for ball1 in 0..(balls.len() - 1) {
        let mut times_ball1: Vec<Option<f64>> = vec![];
        for _ in (ball1 + 1)..balls.len() {
            times_ball1.push(None);
        }
        times.push(times_ball1);
    }
    let mut wall_times: Vec<[Option<f64>; 4]> = vec![[None; 4]; balls.len()];
    let mut image: ImageRGB8 = ImageRGB8::new(width as usize, height as usize, background_color);
    for _ in 0..(fps * video_length) {
        for ball1 in 0..(balls.len() - 1) {
            for ball2 in (ball1 + 1)..balls.len() {
                times[ball1][ball2 - ball1 - 1] = calculate_collision(&mut balls, ball1, ball2);
            }
        }
        for ball in 0..balls.len() {
            for wall in 0..4 {
                wall_times[ball][wall] = calculate_wall_collision(&mut balls, ball, wall, width, height);
            }
        }

        let mut moved_time: f64 = 0.0;
        while moved_time < interval {
            let mut ball_2_ball_col: u8 = 0;  // 0: no collision | 1: ball 2 ball collision | 2: ball 2 wall collision
            let time_left: f64 = interval - moved_time;
            let mut smallest_time: f64 = time_left;
            let mut smallest_ind: [usize; 2] = [0, 0];

            for x in 0..times.len() {
                for y in 0..times[x].len() {
                    if times[x][y].is_some() && times[x][y].unwrap() <= smallest_time {
                        smallest_ind[0] = x;
                        smallest_ind[1] = x + y + 1;
                        smallest_time = times[x][y].unwrap();
                        ball_2_ball_col = 1;
                    }
                }
            }

            for x in 0..wall_times.len() {
                for y in 0..wall_times[x].len() {
                    if wall_times[x][y].is_some() && wall_times[x][y].unwrap() <= smallest_time {
                        smallest_ind[0] = x;
                        smallest_ind[1] = y;
                        smallest_time = wall_times[x][y].unwrap();
                        ball_2_ball_col = 2;
                    }
                }
            }

            match ball_2_ball_col {
                0 => {
                    move_balls(&mut balls, time_left);
                    moved_time += time_left;
                },
                1 => {
                    move_balls(&mut balls, smallest_time);

                    let d = balls[smallest_ind[0]].r + balls[smallest_ind[1]].r;
                    let nx = (balls[smallest_ind[1]].x - balls[smallest_ind[0]].x) / d;
                    let ny = (balls[smallest_ind[1]].y - balls[smallest_ind[0]].y) / d;
                    let p = (2.0 * (nx * (balls[smallest_ind[0]].v_x - balls[smallest_ind[1]].v_x) + ny * (balls[smallest_ind[0]].v_y - balls[smallest_ind[1]].v_y))) / (balls[smallest_ind[0]].m + balls[smallest_ind[1]].m);
                    balls[smallest_ind[0]].v_x -= p * balls[smallest_ind[1]].m * nx;
                    balls[smallest_ind[0]].v_y -= p * balls[smallest_ind[1]].m * ny;
                    balls[smallest_ind[1]].v_x += p * balls[smallest_ind[0]].m * nx;
                    balls[smallest_ind[1]].v_y += p * balls[smallest_ind[0]].m * ny;

                    for x in 0..times.len() {
                        for y in 0..times[x].len() {
                            if smallest_ind.contains(&x) || smallest_ind.contains(&(x + y + 1)) {
                                if x == smallest_ind[0] && (x + y + 1) == smallest_ind[1] {
                                    times[x][y] = None;
                                } else {
                                    times[x][y] = calculate_collision(&mut balls, x, x + y + 1);
                                }
                            } else if times[x][y].is_some() {
                                times[x][y] = Some(times[x][y].unwrap() - smallest_time);
                            }
                        }
                    }

                    for x in 0..wall_times.len() {
                        for y in 0..4 {
                            if smallest_ind.contains(&x) {
                                wall_times[x][y] = calculate_wall_collision(&mut balls, x, y, width, height);
                            } else if wall_times[x][y].is_some() {
                                wall_times[x][y] = Some(wall_times[x][y].unwrap() - smallest_time);
                            }
                        }
                    }

                    moved_time += smallest_time;
                },
                2 => {
                    move_balls(&mut balls, smallest_time);
                    if smallest_ind[1] < 2 {
                        balls[smallest_ind[0]].v_x *= -1.0;
                    } else {
                        balls[smallest_ind[0]].v_y *= -1.0;
                    }

                    for x in 0..times.len() {
                        for y in 0..times[x].len() {
                            if x == smallest_ind[0] || (x + y + 1) == smallest_ind[0] {
                                times[x][y] = calculate_collision(&mut balls, x, x + y + 1);
                            } else if times[x][y].is_some() {
                                times[x][y] = Some(times[x][y].unwrap() - smallest_time);
                            }
                        }
                    }

                    for x in 0..wall_times.len() {
                        for y in 0..4 {
                            if x == smallest_ind[0] {
                                if y == smallest_ind[1] {
                                    wall_times[x][y] = None;
                                } else {
                                    wall_times[x][y] = calculate_wall_collision(&mut balls, x, y, width, height);
                                }
                            } else if wall_times[x][y].is_some() {
                                wall_times[x][y] = Some(wall_times[x][y].unwrap() - smallest_time);
                            }
                        }
                    }

                    moved_time += smallest_time;
                },
                _ => panic!("Invalid collision type"),
            }
        }

        generate_frame(&balls, &mut image);
        stdin.write_all(image.to_bytes()).unwrap();
    }

    let output = ffmpeg_encoder.wait_with_output().unwrap();
    println!("{}", String::from_utf8(output.stdout).unwrap());
    println!("{}", String::from_utf8(output.stderr).unwrap());
}

fn move_balls(balls: &mut Vec<Ball>, interval: f64) {
    for ball in balls {
        ball.x += ball.v_x * interval;
        ball.y += ball.v_y * interval;
    }
}

fn calculate_collision(balls: &mut [Ball], ball1: usize, ball2: usize) -> Option<f64> {
    // write position of balls as functions of time (x + vx*t, y + vy*t)
	// write distance of 2 balls with those functions
	// square to get rid of square root
	// find minimum value of that distance^2 function, and if it is smaller than d^2, find solutions for that function, take the one that happens sooner

    let d_pow2 = (balls[ball1].r + balls[ball2].r).powi(2); // distance between balls at collision squared (d^2)
    let delta_x = balls[ball1].x - balls[ball2].x; // x1 - x2
	let delta_y = balls[ball1].y - balls[ball2].y; // y1 - y2
	let delta_vx = balls[ball1].v_x - balls[ball2].v_x; // vx1 - vx2
	let delta_vy = balls[ball1].v_y - balls[ball2].v_y; // vy1 - vy2

    // calculate coefficients of distance^2 function
	let a = delta_vx.powi(2) + delta_vy.powi(2); // first coefficient
    if a != 0.0 { // if a is 0, then function is not quadratic, balls aren't moving, therefore, there is no collision
        let b_divis_2 = (delta_x * delta_vx) + (delta_y * delta_vy);  // second coefficient divided by 2 (it simplifies function when in that form)
        let c = delta_x.powi(2) + delta_y.powi(2);  // third coefficient

        if (c - (b_divis_2.powi(2) / a)) < d_pow2 { // if minimum value of distance^2 function is smaller than d^2, then the balls would collide
            // find solutions for function, when it's value is d^2
            let discriminant_sqrt = (b_divis_2.powi(2) - (a * (c - d_pow2))).sqrt();
            let mut sol_1: Option<f64> = Some((- b_divis_2 - discriminant_sqrt) / a);
            let mut sol_2: Option<f64> = Some((- b_divis_2 + discriminant_sqrt) / a);
            if sol_1.unwrap() < 0.0 {
                sol_1 = None;
            }
            if sol_2.unwrap() < 0.0 {
                sol_2 = None;
            }

            if sol_1.is_some() {
                return if sol_2.is_some() {
                    Some(sol_1.unwrap().min(sol_2.unwrap()))
                } else {
                    sol_1
                }
            } else if sol_2.is_some() && sol_1.is_none() {
                return sol_2;
            }
        }
    }
    None
}

fn calculate_wall_collision(balls: &mut [Ball], ball: usize, wall: usize, width: f64, height: f64) -> Option<f64> {
    // end position minus start position divided by speed
    let result: f64 = match wall {
        0 => (balls[ball].r - balls[ball].x) / balls[ball].v_x, // left
        1 => (width - balls[ball].r - 1.0 - balls[ball].x) / balls[ball].v_x, // right
        2 => (balls[ball].r - balls[ball].y) / balls[ball].v_y, // bottom
        3 => (height - balls[ball].r - 1.0 - balls[ball].y) / balls[ball].v_y, // top
        _ => panic!("Invalid wall"),
    };
    if result > 0.0 {
        Some(result)
    } else {
        None
    }
}

fn generate_frame(balls: &[Ball], img: &mut ImageRGB8) {
    img.clear();
    for ball in balls {
        img.draw_circle((ball.x).round() as usize, (ball.y).round() as usize, (ball.r).round() as usize, ball.color, 0, 1.0);
    }
}

struct Ball {
    x: f64,
    y: f64,
    m: f64,
    r: f64,
    v_x: f64,
    v_y: f64,
    color: [u8; 3]
}

impl Ball {
    fn new(x: f64, y: f64, mass: f64, radius: f64, velocity_x: f64, velocity_y: f64, color: [u8; 3]) -> Self {
        Self {
            x,
            y,
            m: mass,
            r: radius,
            v_x: velocity_x,
            v_y: velocity_y,
            color
        }
    }
}
