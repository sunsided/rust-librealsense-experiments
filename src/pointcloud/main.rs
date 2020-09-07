mod pcdvizwindow;

use anyhow::Result;
use crossbeam::channel;
use image::{DynamicImage, ImageFormat};
use nalgebra::Point3;
use pcdvizwindow::PcdVizWindow;
use realsense_rust::pipeline::marker::Active;
use realsense_rust::{
    prelude::*, processing_block::marker as processing_block_marker, Config, Error as RsError,
    Format, Pipeline, ProcessingBlock, Resolution, StreamKind,
};
use std::time::Duration;

#[tokio::main]
pub async fn main() -> Result<()> {
    let (tx, rx) = channel::unbounded();

    PcdVizWindow::spawn_new(rx);

    // Initialize the PointCloud filter.
    let mut pointcloud = ProcessingBlock::<processing_block_marker::PointCloud>::create()?;

    // Init the pipeline.
    let mut pipeline = create_pipeline().await?;

    // Show some stream information.
    let profile = pipeline.profile();
    for (idx, stream_result) in profile.streams()?.try_into_iter()?.enumerate() {
        let stream = stream_result?;
        println!("stream data {}: {:#?}", idx, stream.get_data()?);
    }

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

        // save video frame
        {
            let image: DynamicImage = color_frame.image()?.into();
            image.save_with_format(
                format!("sync-video-example-{}.png", color_frame.number()?),
                ImageFormat::Png,
            )?;
        }

        // save depth frame
        {
            let Resolution { width, height } = depth_frame.resolution()?;
            let distance = depth_frame.distance(width / 2, height / 2)?;
            println!("distance = {}", distance);

            let image: DynamicImage = depth_frame.image()?.into();
            image.save_with_format(
                format!("sync-depth-example-{}.png", depth_frame.number()?),
                ImageFormat::Png,
            )?;
        }

        // compute point cloud
        pointcloud.map_to(color_frame.clone())?;
        let points_frame = pointcloud.calculate(depth_frame.clone())?;
        let points = points_frame
            .vertices()?
            .iter()
            .map(|vertex| {
                let [x, y, z] = vertex.xyz;
                Point3::new(x, y, z)
            })
            .collect::<Vec<_>>();

        if tx.send(points).is_err() {
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
