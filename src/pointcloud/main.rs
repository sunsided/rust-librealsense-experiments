mod pcdvizwindow;

use anyhow::Result;
use image::{DynamicImage, ImageFormat};
use nalgebra::Point3;
use pcdvizwindow::PcdVizWindow;
use realsense_rust::frame::marker::Depth;
use realsense_rust::kind::Extension::Points;
use realsense_rust::processing_block::marker::PointCloud;
use realsense_rust::{
    frame::marker::Video, pipeline::marker::Active, prelude::*,
    processing_block::marker as processing_block_marker, stream_profile::*, Config,
    Error as RsError, Format, Frame, Pipeline, ProcessingBlock, Resolution, StreamKind,
    StreamProfile,
};
use std::time::Duration;

#[tokio::main]
pub async fn main() -> Result<()> {
    let window = PcdVizWindow::spawn_new();

    // Initialize the PointCloud filter.
    let mut pointcloud = ProcessingBlock::<processing_block_marker::PointCloud>::create()?;

    // Init the pipeline and show the profile information of its streams.
    let mut pipeline = create_pipeline().await?;
    show_profiles(&pipeline)?;

    // Process the frames.
    for _ in 0usize..1000 {
        let timeout = Duration::from_millis(1000);
        let frames_result = pipeline.wait_async(Some(timeout)).await;
        let frames = match frames_result {
            Err(RsError::Timeout(..)) => {
                println!("timeout error");
                continue;
            }
            result => result?,
        };

        println!("frame number = {}", frames.number()?);

        let color_frame = frames.color_frame()?.unwrap();
        let depth_frame = frames.depth_frame()?.unwrap();

        // Save the frames for inspection.
        //        save_video_frame(&color_frame)?;
        //        save_depth_frame(&depth_frame)?;

        // Compute and visualize the point cloud.
        let points = process_point_cloud(&mut pointcloud, color_frame, depth_frame)?;
        if window.update(points).is_err() {
            break;
        }
    }

    Ok(())
}

async fn create_pipeline() -> Result<Pipeline<Active>> {
    let pipeline = Pipeline::new()?;
    let config = Config::new()?
        .enable_stream(StreamKind::Depth, 0, 640, 0, Format::Z16, 30)?
        .enable_stream(StreamKind::Color, 0, 640, 0, Format::Rgb8, 30)?;
    let pipeline = pipeline.start_async(Some(config)).await?;
    Ok(pipeline)
}

fn show_profiles(pipeline: &Pipeline<Active>) -> Result<()> {
    let profile = pipeline.profile();
    for (idx, stream_result) in profile.streams()?.try_into_iter()?.enumerate() {
        let stream = stream_result?;
        println!("stream data {}: {:#?}", idx, stream.get_data()?);
    }
    Ok(())
}

fn save_video_frame(color_frame: &Frame<Video>) -> Result<()> {
    let image: DynamicImage = color_frame.image()?.into();
    image.save_with_format(
        format!("sync-video-example-{}.png", color_frame.number()?),
        ImageFormat::Png,
    )?;
    Ok(())
}

fn save_depth_frame(depth_frame: &Frame<Depth>) -> Result<()> {
    let Resolution { width, height } = depth_frame.resolution()?;
    let distance = depth_frame.distance(width / 2, height / 2)?;
    println!("distance = {}", distance);

    let image: DynamicImage = depth_frame.image()?.into();
    image.save_with_format(
        format!("sync-depth-example-{}.png", depth_frame.number()?),
        ImageFormat::Png,
    )?;

    Ok(())
}

fn process_point_cloud(
    pointcloud: &mut ProcessingBlock<PointCloud>,
    color_frame: Frame<Video>,
    depth_frame: Frame<Depth>,
) -> Result<Vec<(Point3<f32>, Point3<f32>)>> {
    pointcloud.map_to(color_frame.clone())?;
    let points_frame = pointcloud.calculate(depth_frame.clone())?;

    let vertices = points_frame.vertices()?;
    let pixels = points_frame.texture_coordinates()?;

    let points = vertices
        .iter()
        .zip(pixels.iter())
        .map(|(vertex, pixel)| {
            let [x, y, z] = vertex.xyz;
            let xyz = Point3::new(x, y, z);

            let (r, g, b) = get_texcolor(&color_frame, &pixel.ij).expect("tex coords invalid");
            let rgb = Point3::new(r, g, b);
            (xyz, rgb)
        })
        .collect::<Vec<_>>();
    Ok(points)
}

fn get_texcolor(texture: &Frame<Video>, [u, v]: &[i32; 2]) -> Result<(f32, f32, f32)> {
    let w = texture.width()?;
    let h = texture.height()?;
    let data = texture.data()?;
    let bytes_per_pixel = texture.bits_per_pixel()? / 8;
    let stride = texture.stride_in_bytes()?;

    // https://github.com/IntelRealSense/librealsense/issues/6234
    // From https://github.com/IntelRealSense/librealsense/issues/6234#issuecomment-613352862:
    //   The Field of View (FOV) of the Depth sensor is bigger than the RGB sensor's, hence while
    //   the sensors overlap you can't have RGB data coverage on the boundaries of the depth frame.
    //   The [U,V] outliers designate pixels for which the texture mapping occurs outside of the
    //   RGB sensor's FOV. #2355
    let x_raw = unsafe { std::mem::transmute::<i32, f32>(*u) };
    let y_raw = unsafe { std::mem::transmute::<i32, f32>(*v) };

    let x = scale_and_clamp(x_raw, w);
    let y = scale_and_clamp(y_raw, h);

    let (x, y) = match (x, y) {
        (Some(x), Some(y)) => (x, y),
        _ => return Ok((0f32, 0f32, 0f32)),
    };

    let idx = x * bytes_per_pixel + y * stride;
    let r = data[idx] as f32 / 255f32;
    let g = data[idx + 1] as f32 / 255f32;
    let b = data[idx + 2] as f32 / 255f32;
    Ok((r, g, b))
}

/// Scales the provided coordinate to a range of `0`..`max_value`. If the input
/// coordinate is outside the range `0..1`, the value `None` is returned.
fn scale_and_clamp(coordinate: f32, max_value: usize) -> Option<usize> {
    if coordinate < 0f32 || coordinate >= 1f32 {
        return None;
    }

    Some(std::cmp::min(
        std::cmp::max((coordinate * max_value as f32) as usize, 0),
        max_value as usize - 1,
    ) as usize)
}
