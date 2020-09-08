# Point Cloud Processing

Using the `ProcessingBlock<processing_block_marker::PointCloud>` pipeline
to obtain depth data from a RealSense D435, map texture coordinates to the
vertices, then render it in 3D.

![](.readme/texture-mapping.webp)

## Texture mapping the vertices

In an attempt to add colored pixels to the example code of 
[realsense-rust](https://github.com/jerry73204/realsense-rust) 0.3.2,
I stumbled over an issue with the texture coordinates. Specifically,
the [Frame](https://docs.rs/realsense-rust/0.3.2/realsense_rust/frame/struct.Frame.html) method [texture_coordinates()](https://docs.rs/realsense-rust/0.3.2/realsense_rust/frame/struct.Frame.html#method.texture_coordinates)
returns an [rs2_pixel](https://docs.rs/realsense-sys/0.2.4/realsense_sys/struct.rs2_pixel.html), which is
labeled as `[c_int; 2]` for UV coordinates. At runtime these values all fluctuate wildly between negative and positive millions.

The values didn't seem to make a lot of sense to me even after applying a gentle bit of empirical
science (i.e. poking things with a stick) until I stumbled over
[this comment](https://github.com/IntelRealSense/librealsense/issues/6234#issuecomment-613352862) in 
librealsense issue [#6234](https://github.com/IntelRealSense/librealsense/issues/6234):

> The Field of View (FOV) of the Depth sensor is bigger than the RGB sensor's, hence while the sensors overlap you can't have RGB data coverage on the boundaries of the depth frame.

In that issue, the author observed that the value range of the texture coordinates
was in the range of `-0.2 .. 1.2` when it should have been in range `0 .. 1`.
The only way to arrive at a similar value range was by transmuting the `[i32; 2]`
coordinates into `[f32; 2]` coordinates:

```rust
let x_raw = unsafe { std::mem::transmute::<i32, f32>(*u) };
let y_raw = unsafe { std::mem::transmute::<i32, f32>(*v) };
```

After that, discarding everything out of range `0 .. 1` (i.e. mapping it to black) resulted in correct pixels.
It appears that the official documentation indeed returns an [rs2::texture_coordinate](https://intelrealsense.github.io/librealsense/doxygen/structrs2_1_1texture__coordinate.html)
struct instead which is composed of two `float` values, but there seems to be
a discrepancy between the C++ API (returning floats) and the C API (returning ints).
I opened issue [realsense-rust #14](https://github.com/jerry73204/realsense-rust/issues/14) about it.

---

![](.readme/texture-mapping.png)
