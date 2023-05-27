#![allow(unused_imports)]
#[macro_use] extern crate rocket;
extern crate ndarray;
extern crate rand;
extern crate plotters;

//Webserver imports
use rocket::response::content;
use rocket::fs::FileServer;
use rocket::response::content::RawHtml;
use rocket::serde::{Serialize, Deserialize};
use rocket::serde::json::serde_json::Value;
use rocket::serde::json::Json;

//Plotting
use plotters::prelude::*;
//use plotchart::drawing::bitmap_pixel::BGRXPixel;
use image::{ImageFormat, ImageEncoder};
use image::codecs::png::PngEncoder;
use image::{RgbaImage, Rgba};
use plotters::backend::BGRXPixel;

//Encoding
use base64::{Engine as _, engine::general_purpose};
use std::io::Cursor;

use ndarray::Array;
use rand::Rng;
use std::fs::File;
use std::path::Path;



// Generate a fractional Brownian motion with Hurst parameter h
// n: number of points
// h: Hurst parameter
fn generate_fbm(n: usize, h: f64) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let mut fbm = Vec::with_capacity(n);
    fbm.push(0.0);
    for i in 1..n {
        let mut sum = 0.0;
        for j in 0..i {
            sum += (i - j) as f64 * (j as f64).powf(h - 0.5) * (rng.gen::<f64>() - 0.5);
        }
        fbm.push(sum);
    }
    fbm
}

fn get_limits_of_fbm(fbm: &Vec<f64>) -> (f64, f64) {
    let mut min = 0.0;
    let mut max = 0.0;
    for i in 0..fbm.len() {
        if fbm[i] < min {
            min = fbm[i];
        }
        if fbm[i] > max {
            max = fbm[i];
        }
    }
    (min, max)
}

//Function to plot the output of fbm as a time series and return as a png and writes to dist
fn plot_fbm_to_png(fbm: Vec<f64>, dist: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (min, max) = get_limits_of_fbm(&fbm);
    let root = BitMapBackend::new(dist, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Fractional Brownian Motion", ("sans-serif", 50))
        .margin(20)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..fbm.len(), min..max)?;
    chart.configure_mesh().draw()?;
    chart.draw_series(LineSeries::new(
        (0..fbm.len()).map(|i| (i, fbm[i])),
        &RED,
    ))?;
    Ok(())
}


//Pake in a vector of f64 and plot it
fn plot_fbm(fbm: Vec<f64>) -> Result<image::RgbImage, Box<dyn std::error::Error>> {
    let (min, max) = get_limits_of_fbm(&fbm);
    let (width, height) = (1024, 768);

    let mut imgbuf = image::RgbImage::new(width, height);
    {
        let root = BitMapBackend::with_buffer(&mut imgbuf, (width, height))
            .into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption("Fractional Brownian Motion", ("sans-serif", 50))
            .margin(20)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0..fbm.len(), min..max)?;
        chart.configure_mesh().draw()?;
        chart.draw_series(LineSeries::new(
            (0..fbm.len()).map(|i| (i, fbm[i])),
            &RED,
        ))?;
    }

    // Save the image to a file as PNG before returning it
    imgbuf.save_with_format("fbm.png", image::ImageFormat::Png)?;

    Ok(imgbuf)
}


#[allow(dead_code)]
fn return_image() -> RawHtml<String> {
    RawHtml("<img src=\"fbm.png\" alt=\"fbm\">".to_string())
}

// Define the handler for the /fbm endpoint
#[get("/fbm/<n>/<h>")]
fn fbm(n: usize, h: f64) -> RawHtml<String> {
    let fbm = generate_fbm(n,h);
    let img: image::RgbImage = plot_fbm(fbm).expect("Unable to build image from data.");
    let img: image::DynamicImage = image::DynamicImage::ImageRgb8(img);

    // Convert the encoded PNG data to a base64 string
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png).unwrap();
    let img_base64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(&buf);

    // Create a data URL and serve it
    let data_url = format!("data:image/png;base64,{}", img_base64);
    RawHtml(format!("<img src=\"{}\" alt=\"fbm\">", data_url))
}


// Launch the server
#[launch]
fn rocket() -> _ {
    let fbm = generate_fbm(1000, 0.5);
    plot_fbm_to_png(fbm, "fbm.png").expect("Unable to plot FBM to PNG.");
    rocket::build().mount("/", routes![fbm])
}