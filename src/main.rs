#![allow(non_snake_case)]

use tempfile::tempdir;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use tinydraw::ImageRGB8;

fn main() {
    let bundled_ffmpeg = include_bytes!("../data/ffmpeg-compressed.exe");

    let dir = tempdir().unwrap();
    let ffmpeg_path = dir.path().join("ffmpeg.exe");
    let mut file = File::create(&ffmpeg_path).unwrap();
    file.write_all(bundled_ffmpeg).unwrap();
    drop(file);

    simulation(ffmpeg_path.to_str().unwrap());

    drop(dir);
}

fn simulation(ffmpeg_path: &str) {
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
