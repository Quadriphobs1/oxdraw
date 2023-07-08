//! Graphics API renderer for app

mod error;

use std::collections::HashMap;

use anyhow::bail;
use piet_common::{
    kurbo::{BezPath, Point, Rect, Size},
    BitmapTarget, Color, Device, FontFamily, ImageFormat, InterpolationMode, RenderContext, Text,
    TextLayoutBuilder,
};
use pixels::{
    raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle},
    Pixels, SurfaceTexture,
};

use crate::error::RendererError;

/// The Renderer struct holds a Device and an optional Pixels buffer.
/// The Device is used to interface with the graphics API (for example `OpenGL`) and perform rendering operations.
/// A Pixels buffer represents a block of pixel data in memory that can be uploaded to the GPU. Before rendering a frame, a new Pixels buffer is created to hold the pixel data for that frame. Then, the Pixels buffer is passed to the Device to be rendered.
pub struct Renderer {
    /// device is used to interface with the graphics API (for example OpenGL) and perform rendering operations.
    device: Device,
    /// A Pixels buffer represents a block of pixel data in memory that can be uploaded to the GPU. Before rendering a frame, a new Pixels buffer is created to hold the pixel data for that frame. Then, the Pixels buffer is passed to the Device to be rendered.
    pixel_buffers: HashMap<u64, Pixels>,
}

/// Constructor functions
impl Renderer {
    pub fn new() -> anyhow::Result<Renderer> {
        // piet_common::Device does not implement `Send + Sync` because not all the backend can support `Sync`
        // so we handle the error case manually to avoid error casting when using `?`
        match Device::new() {
            Ok(device) => Ok(Renderer {
                device,
                pixel_buffers: HashMap::new(),
            }),
            Err(err) => bail!(RendererError::RendererDevice(err.to_string())),
        }
    }
}

/// Mutable functions
impl Renderer {
    /// Create and store a new pixel buffer for the window
    pub fn setup<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &mut self,
        window_id: u64,
        width: u32,
        height: u32,
        window: &W,
    ) -> anyhow::Result<()> {
        let surface_texture = SurfaceTexture::new(width, height, &window);
        let pixels = Pixels::new(width, height, surface_texture)?;
        self.pixel_buffers.insert(window_id, pixels);

        Ok(())
    }

    pub fn update(&mut self, window_id: u64, width: u32, height: u32) -> anyhow::Result<()> {
        let pixel = match self.pixel_buffers.get_mut(&window_id) {
            None => bail!(RendererError::MissingPixelBuffer(window_id)),
            Some(p) => p,
        };

        pixel.resize_surface(width, height)?;
        pixel.resize_buffer(width, height)?;

        Ok(())
    }

    pub fn remove(&mut self, window_id: u64) -> anyhow::Result<()> {
        self.pixel_buffers.remove(&window_id);
        Ok(())
    }

    pub fn render(
        &mut self,
        window_id: u64,
        width: u32,
        height: u32,
        scale_factor: f64,
    ) -> anyhow::Result<()> {
        let pixel = match self.pixel_buffers.get_mut(&window_id) {
            None => bail!(RendererError::MissingPixelBuffer(window_id)),
            Some(p) => p,
        };
        let mut bitmap = self
            .device
            .bitmap_target(width as usize, height as usize, scale_factor)
            .unwrap();

        draw_canvas(&mut bitmap, width, height);
        let buffer = pixel.frame_mut();
        bitmap
            .copy_raw_pixels(piet_common::ImageFormat::RgbaPremul, buffer)
            .ok();

        if let Err(err) = pixel.render() {
            bail!(RendererError::FailedRender(err))
        }
        Ok(())
    }
}

fn draw_canvas(bitmap: &mut BitmapTarget<'_>, width: u32, height: u32) {
    let mut ctx = bitmap.render_context();

    let data = "Hello from Piet + Winit";
    let size = Size::new(width as f64, height as f64);
    let rect = size.to_rect();
    ctx.fill(rect, &Color::WHITE);

    let mut path = BezPath::new();
    path.move_to((0.0, size.height));
    path.quad_to((40.0, 50.0), (size.width, 0.0));

    // Create a color
    let stroke_color = Color::rgb8(128, 0, 0);
    // Stroke the path with thickness 1.0
    ctx.stroke(path, &stroke_color, 15.0);

    // Create an arbitrary bezier path
    let mut path = BezPath::new();
    path.move_to(Point::ORIGIN);
    path.quad_to((40.0, 50.0), (size.width, size.height));
    // Create a color
    let stroke_color = Color::rgb8(0, 128, 0);
    // Stroke the path with thickness 5.0
    ctx.stroke(path, &stroke_color, 5.0);

    // Rectangles: the path for practical people
    let rect = Rect::from_origin_size((400.0, 20.0), (100.0, 100.0));
    // Note the Color:rgba8 which includes an alpha channel (7F in this case)
    let fill_color = Color::rgba8(0x00, 0x00, 0x00, 0x7F);
    ctx.fill(rect, &fill_color);

    // draw test
    let text = ctx.text();
    let layout = text
        .new_text_layout(data)
        .font(FontFamily::SANS_SERIF, 24.0)
        .max_width(400.0)
        .text_color(Color::rgb8(128, 0, 0))
        .build()
        .unwrap();
    ctx.draw_text(&layout, (300.0, 200.0));

    // Let's burn some CPU to make a (partially transparent) image buffer
    let image_data = make_image_data(256, 256);
    let image = ctx
        .make_image(256, 256, &image_data, ImageFormat::RgbaSeparate)
        .unwrap();
    // The image is automatically scaled to fit the rect you pass to draw_image
    ctx.draw_image(&image, size.to_rect(), InterpolationMode::Bilinear);

    ctx.finish().ok();
}

fn make_image_data(width: usize, height: usize) -> Vec<u8> {
    let mut result = vec![0; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let ix = (y * width + x) * 4;
            result[ix] = x as u8;
            result[ix + 1] = y as u8;
            result[ix + 2] = !(x as u8);
            result[ix + 3] = 127;
        }
    }
    result
}
