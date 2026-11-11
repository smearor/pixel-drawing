//! Interactive demo for pixel-drawing.
//!
//! Demonstrates all major rendering capabilities of the library:
//! - `fill_background` — solid color fill
//! - `draw_nerd_font_icon` — icon rendering by name
//! - `draw_icon_placeholder` — fallback placeholder
//! - `draw_nerd_font_codepoint` — icon rendering by codepoint
//! - `draw_icon_grid` — multi-icon grid layout
//! - `draw_progress_bar` — progress bar rendering
//! - `draw_label_text` — bottom label text
//! - `draw_text_centered` — centered text at arbitrary position
//!
//! Run with:
//! ```sh
//! cargo run --example interactive_demo
//! ```

use nerd_fonts_rs::{IconName};
use nerd_fonts_rs::{init};
use pixel_drawing::color::Color;
use pixel_drawing::pixel_buffer::PixelBuffer;
use pixel_drawing::render_context::RenderContext;
use pixel_drawing::size::Size;

use gtk4::Align;
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::Box;
use gtk4::Frame;
use gtk4::Image;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::Scale;
use gtk4::glib;
use gtk4::prelude::*;
use miette::IntoDiagnostic;
use miette::Result;

const APP_ID: &str = "io.smearor.pixel_drawing.interactive_demo";

/// A curated selection of Nerd Font icons to display.
const GALLERY_ICONS: &[&str] = &[
    "nf-fa-gamepad",
    "nf-md-cube",
    "nf-linux-tux",
    "nf-fa-star",
    "nf-md-cog",
    "nf-fa-heart",
    "nf-md-battery",
    "nf-fa-music",
];

fn main() -> Result<glib::ExitCode> {
    gtk4::init().into_diagnostic()?;
    init().into_diagnostic()?;

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    Ok(app.run())
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("pixel-drawing Interactive Demo")
        .default_width(900)
        .default_height(900)
        .build();

    window.connect_close_request({
        let app = app.clone();
        move |_| {
            app.quit();
            glib::Propagation::Proceed
        }
    });

    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let title = Label::builder()
        .label("pixel-drawing Interactive Demo")
        .css_classes(["title-1"])
        .halign(Align::Center)
        .build();
    main_box.append(&title);

    let hint = Label::builder()
        .label("Software rendering of icons, text, and progress bars onto raw RGBA pixel buffers.")
        .halign(Align::Center)
        .css_classes(["dim-label"])
        .build();
    main_box.append(&hint);

    main_box.append(&build_fill_background_section());
    main_box.append(&build_icon_rendering_section());
    main_box.append(&build_codepoint_section());
    main_box.append(&build_icon_grid_section());
    main_box.append(&build_progress_bar_section());
    main_box.append(&build_text_section());
    main_box.append(&build_render_context_section());

    let scrolled = gtk4::ScrolledWindow::builder().child(&main_box).hexpand(true).vexpand(true).build();

    window.set_child(Some(&scrolled));
    window.present();
}

// ---------------------------------------------------------------------------
// Helper: resolve icon name to codepoint
// ---------------------------------------------------------------------------

fn resolve_codepoint(name: &str) -> Option<char> {
    IconName::from_glyph_name(name.strip_prefix("nf-").unwrap_or(name))
        .and_then(|n| n.codepoint())
        .map(|c| c.as_char())
}

// ---------------------------------------------------------------------------
// Helper: convert pixel buffer to GTK texture
// ---------------------------------------------------------------------------

fn pixels_to_texture(buf: &PixelBuffer) -> gtk4::gdk::MemoryTexture {
    let size = buf.size();
    let bytes = glib::Bytes::from(buf.pixels());
    let rowstride = size.width() * 4;
    gtk4::gdk::MemoryTexture::new(size.width() as i32, size.height() as i32, gtk4::gdk::MemoryFormat::R8g8b8a8, &bytes, rowstride as usize)
}

fn make_image(texture: &gtk4::gdk::MemoryTexture, size: u32) -> Image {
    let image = Image::from_paintable(Some(texture));
    image.set_halign(Align::Center);
    image.set_pixel_size(size as i32);
    image.set_width_request(size as i32);
    image.set_height_request(size as i32);
    image
}

// ---------------------------------------------------------------------------
// Section 1: fill_background
// ---------------------------------------------------------------------------

fn build_fill_background_section() -> Frame {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let header = Label::builder()
        .label("1. fill_background")
        .css_classes(["title-3"])
        .halign(Align::Start)
        .build();
    section.append(&header);

    let desc = Label::builder()
        .label("Fills an RGBA pixel buffer with a solid color.")
        .halign(Align::Start)
        .wrap(true)
        .build();
    section.append(&desc);

    let size = Size::new(64, 64);
    let mut buf = PixelBuffer::new(size);
    buf.fill_background(Color::<f64>::new(0.2, 0.5, 0.8));

    let texture = pixels_to_texture(&buf);
    let image = make_image(&texture, size.width());
    section.append(&image);

    Frame::builder().label("fill_background").child(&section).build()
}

// ---------------------------------------------------------------------------
// Section 2: draw_nerd_font_icon + draw_icon_placeholder
// ---------------------------------------------------------------------------

fn build_icon_rendering_section() -> Frame {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let header = Label::builder()
        .label("2. draw_nerd_font_icon & draw_icon_placeholder")
        .css_classes(["title-3"])
        .halign(Align::Start)
        .build();
    section.append(&header);

    let desc = Label::builder()
        .label("Renders Nerd Font icons by name onto a pixel buffer. Unknown icons fall back to a circular placeholder.")
        .halign(Align::Start)
        .wrap(true)
        .build();
    section.append(&desc);

    let grid = gtk4::Grid::builder()
        .column_spacing(12)
        .row_spacing(12)
        .halign(Align::Center)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let cols = 4u32;
    let icon_size = Size::new(96, 96);

    for (i, icon_name) in GALLERY_ICONS.iter().enumerate() {
        let mut buf = PixelBuffer::new(icon_size);
        buf.fill_background(Color::<f64>::new(30.0 / 255.0, 30.0 / 255.0, 30.0 / 255.0));
        buf.draw_nerd_font_icon(icon_name, true, resolve_codepoint, Some(Color::<f64>::new(0.4, 0.7, 1.0)));
        buf.draw_label_text(&icon_name.replace("nf-", ""), false, None);

        let texture = pixels_to_texture(&buf);
        let image = make_image(&texture, icon_size.width());

        let col = (i as u32) % cols;
        let row = (i as u32) / cols;
        grid.attach(&image, col as i32, row as i32, 1, 1);
    }

    // Draw a placeholder for an unknown icon
    let mut placeholder_buf = PixelBuffer::new(icon_size);
    placeholder_buf.fill_background(Color::<f64>::new(30.0 / 255.0, 30.0 / 255.0, 30.0 / 255.0));
    placeholder_buf.draw_icon_placeholder(false);

    let placeholder_texture = pixels_to_texture(&placeholder_buf);
    let placeholder_image = make_image(&placeholder_texture, icon_size.width());
    grid.attach(&placeholder_image, cols as i32, 0, 1, 1);

    section.append(&grid);

    Frame::builder().label("Icon Rendering").child(&section).build()
}

// ---------------------------------------------------------------------------
// Section 3: draw_nerd_font_codepoint
// ---------------------------------------------------------------------------

fn build_codepoint_section() -> Frame {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let header = Label::builder()
        .label("3. draw_nerd_font_codepoint")
        .css_classes(["title-3"])
        .halign(Align::Start)
        .build();
    section.append(&header);

    let desc = Label::builder()
        .label("Renders a Nerd Font icon by its Unicode codepoint at a custom position and size.")
        .halign(Align::Start)
        .wrap(true)
        .build();
    section.append(&desc);

    let size = Size::new(96, 96);
    let mut buf = PixelBuffer::new(size);
    buf.fill_background(Color::<f64>::new(25.0 / 255.0, 25.0 / 255.0, 35.0 / 255.0));
    buf.draw_nerd_font_codepoint('\u{F11B}', size.width() as f32 / 2.0, size.height() as f32 / 2.0, 48.0, Color::<u8>::new(100, 180, 255));

    let texture = pixels_to_texture(&buf);
    let image = make_image(&texture, size.width());
    section.append(&image);

    Frame::builder().label("Codepoint Rendering").child(&section).build()
}

// ---------------------------------------------------------------------------
// Section 4: draw_icon_grid
// ---------------------------------------------------------------------------

fn build_icon_grid_section() -> Frame {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let header = Label::builder()
        .label("4. draw_icon_grid")
        .css_classes(["title-3"])
        .halign(Align::Start)
        .build();
    section.append(&header);

    let desc = Label::builder()
        .label("Renders a grid of icons within a single pixel buffer.")
        .halign(Align::Start)
        .wrap(true)
        .build();
    section.append(&desc);

    let size = Size::new(192, 192);
    let mut buf = PixelBuffer::new(size);
    buf.fill_background(Color::<f64>::new(20.0 / 255.0, 20.0 / 255.0, 25.0 / 255.0));
    buf.draw_icon_grid(GALLERY_ICONS, 2, true, resolve_codepoint);

    let texture = pixels_to_texture(&buf);
    let image = make_image(&texture, size.width());
    section.append(&image);

    Frame::builder().label("Icon Grid").child(&section).build()
}

// ---------------------------------------------------------------------------
// Section 5: draw_progress_bar (interactive)
// ---------------------------------------------------------------------------

fn build_progress_bar_section() -> Frame {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let header = Label::builder()
        .label("5. draw_progress_bar")
        .css_classes(["title-3"])
        .halign(Align::Start)
        .build();
    section.append(&header);

    let desc = Label::builder()
        .label("Renders a horizontal progress bar. Drag the slider to update.")
        .halign(Align::Start)
        .wrap(true)
        .build();
    section.append(&desc);

    let progress_size = Size::new(256, 32);

    let render_progress = move |value: f32| -> gtk4::gdk::MemoryTexture {
        let mut buf = PixelBuffer::new(progress_size);
        buf.fill_background(Color::<f64>::new(20.0 / 255.0, 20.0 / 255.0, 20.0 / 255.0));
        buf.draw_progress_bar(value, Color::<f64>::new(0.4, 0.7, 1.0));
        pixels_to_texture(&buf)
    };

    let progress_texture = render_progress(0.65);
    let progress_image = Image::from_paintable(Some(&progress_texture));
    progress_image.set_halign(Align::Center);
    progress_image.set_pixel_size(progress_size.width() as i32);
    progress_image.set_width_request(progress_size.width() as i32);
    progress_image.set_height_request(progress_size.height() as i32);
    section.append(&progress_image);

    let progress_slider = Scale::with_range(Orientation::Horizontal, 0.0, 1.0, 0.01);
    progress_slider.set_value(0.65);
    progress_slider.set_halign(Align::Center);
    progress_slider.set_width_request(256);
    section.append(&progress_slider);

    progress_slider.connect_value_changed(glib::clone!(
        #[weak]
        progress_image,
        move |slider| {
            let texture = render_progress(slider.value() as f32);
            progress_image.set_paintable(Some(&texture));
        }
    ));

    Frame::builder().label("Progress Bar").child(&section).build()
}

// ---------------------------------------------------------------------------
// Section 6: draw_label_text & draw_text_centered
// ---------------------------------------------------------------------------

fn build_text_section() -> Frame {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let header = Label::builder()
        .label("6. draw_label_text & draw_text_centered")
        .css_classes(["title-3"])
        .halign(Align::Start)
        .build();
    section.append(&header);

    let desc = Label::builder()
        .label("Renders text labels at the bottom of an image and centered text at arbitrary positions.")
        .halign(Align::Start)
        .wrap(true)
        .build();
    section.append(&desc);

    let row = Box::builder().orientation(Orientation::Horizontal).spacing(12).halign(Align::Center).build();

    // draw_label_text
    let label_size = Size::new(96, 96);
    let mut label_buf = PixelBuffer::new(label_size);
    label_buf.fill_background(Color::<f64>::new(30.0 / 255.0, 30.0 / 255.0, 30.0 / 255.0));
    label_buf.draw_nerd_font_icon("nf-fa-star", true, resolve_codepoint, Some(Color::<f64>::new(1.0, 0.8, 0.2)));
    label_buf.draw_label_text("star", true, None);

    let label_texture = pixels_to_texture(&label_buf);
    let label_image = make_image(&label_texture, label_size.width());
    row.append(&label_image);

    // draw_text_centered
    let text_size = Size::new(128, 128);
    let mut text_buf = PixelBuffer::new(text_size);
    text_buf.fill_background(Color::<f64>::new(25.0 / 255.0, 25.0 / 255.0, 35.0 / 255.0));
    text_buf.draw_text_centered("Centered", text_size.height() as f32 / 2.0, 20.0, Color::<u8>::new(255, 255, 255));
    text_buf.draw_text_centered("Text", text_size.height() as f32 / 2.0 + 24.0, 16.0, Color::<u8>::new(100, 180, 255));

    let text_texture = pixels_to_texture(&text_buf);
    let text_image = make_image(&text_texture, text_size.width());
    row.append(&text_image);

    section.append(&row);

    Frame::builder().label("Text Rendering").child(&section).build()
}

// ---------------------------------------------------------------------------
// Section 7: RenderContext (builder pattern)
// ---------------------------------------------------------------------------

fn build_render_context_section() -> Frame {
    let section = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let header = Label::builder()
        .label("7. RenderContext (builder pattern)")
        .css_classes(["title-3"])
        .halign(Align::Start)
        .build();
    section.append(&header);

    let desc = Label::builder()
        .label("Combines fill_background, draw_icon, and draw_label into a single builder-pattern call.")
        .halign(Align::Start)
        .wrap(true)
        .build();
    section.append(&desc);

    let size = Size::new(96, 96);
    let mut buf = PixelBuffer::new(size);

    let mut ctx = RenderContext::new(&mut buf)
        .background(Color::<f64>::new(30.0 / 255.0, 30.0 / 255.0, 30.0 / 255.0))
        .icon_color(Color::<f64>::new(0.9, 0.3, 0.4))
        .active(true);
    ctx.fill_background();
    ctx.draw_icon_by_name("nf-fa-heart", resolve_codepoint);
    ctx.draw_label("heart");

    let texture = pixels_to_texture(&buf);
    let image = make_image(&texture, size.width());
    section.append(&image);

    Frame::builder().label("RenderContext").child(&section).build()
}
