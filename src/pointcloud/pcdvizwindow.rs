use anyhow::Result;
use crossbeam::channel;
use kiss3d::{
    light::Light,
    window::{State, Window},
};
use nalgebra::Point3;

#[derive(Debug)]
pub struct PcdVizWindow {
    tx: channel::Sender<Vec<Point3<f32>>>,
}

impl PcdVizWindow {
    #[must_use]
    pub fn spawn_new() -> Self {
        let (tx, rx) = channel::unbounded();
        let state = PcdVizState::new(rx);

        std::thread::spawn(move || {
            let mut window = Window::new("point cloud");
            window.set_light(Light::StickToCamera);
            window.render_loop(state);
        });

        Self { tx }
    }

    pub fn update(&self, points: Vec<Point3<f32>>) -> Result<()> {
        self.tx.send(points)?;
        Ok(())
    }
}

#[derive(Debug)]
struct PcdVizState {
    rx: channel::Receiver<Vec<Point3<f32>>>,
    points: Option<Vec<Point3<f32>>>,
}

impl PcdVizState {
    pub fn new(rx: channel::Receiver<Vec<Point3<f32>>>) -> Self {
        Self { rx, points: None }
    }
}

impl State for PcdVizState {
    fn step(&mut self, window: &mut Window) {
        // try to receive recent points
        if let Ok(points) = self.rx.try_recv() {
            self.points = Some(points);
        };

        // draw axis
        window.draw_line(
            &Point3::origin(),
            &Point3::new(1.0, 0.0, 0.0),
            &Point3::new(1.0, 0.0, 0.0),
        );
        window.draw_line(
            &Point3::origin(),
            &Point3::new(0.0, 1.0, 0.0),
            &Point3::new(0.0, 1.0, 0.0),
        );
        window.draw_line(
            &Point3::origin(),
            &Point3::new(0.0, 0.0, 1.0),
            &Point3::new(0.0, 0.0, 1.0),
        );

        // draw points
        if let Some(points) = &self.points {
            for point in points.iter() {
                window.draw_point(point, &Point3::new(1.0, 1.0, 1.0));
            }
        }
    }
}
