mod pcdvizwindow;

use anyhow::Result;
use image::{DynamicImage, ImageFormat};
use nalgebra::Point3;
use pcdvizwindow::PcdVizWindow;
use realsense_rust::frame::marker::Depth;
use realsense_rust::processing_block::marker::PointCloud;
use realsense_rust::{
    frame::marker::Video, pipeline::marker::Active, prelude::*,
    processing_block::marker as processing_block_marker, Config, Error as RsError, Format, Frame,
    Pipeline, ProcessingBlock, Resolution, StreamKind,
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
        save_video_frame(&color_frame)?;
        save_depth_frame(&depth_frame)?;

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
) -> Result<Vec<Point3<f32>>> {
    pointcloud.map_to(color_frame)?;
    let points_frame = pointcloud.calculate(depth_frame)?;
    let points = points_frame
        .vertices()?
        .iter()
        .map(|vertex| {
            let [x, y, z] = vertex.xyz;
            Point3::new(x, y, z)
        })
        .collect::<Vec<_>>();
    Ok(points)
}
