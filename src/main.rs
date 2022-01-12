mod canvas;

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use canvas::{color, mxcfb_rect, Canvas, Point2, Vector2};
use libremarkable::framebuffer::FramebufferDraw;

const CELL_SIZE: u32 = 300;
const CELL_MARGIN: u32 = 4;

fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    env_logger::builder().format_timestamp_millis().init();

    let mut canvas = Canvas::new();
    canvas.clear();
    canvas.update_full();

    draw_background(&mut canvas);

    demo_endless(&mut canvas)?;

    info!("Bye!");
    Ok(())
}

fn demo_endless(canvas: &mut Canvas) -> Result<()> {
    let mut i = 0;
    loop {
        let mut num = if i % 2 == 0 { 2 } else { 65536 };
        let dur = std::time::Duration::from_millis(25);
        std::thread::sleep(dur);
        draw_cell(canvas, 0, 0, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 1, 0, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 2, 0, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 3, 0, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 0, 1, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 1, 1, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 2, 1, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 3, 1, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 0, 2, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 1, 2, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 2, 2, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 3, 2, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 0, 3, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 1, 3, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 2, 3, num)?;
        #[rustfmt::skip] if i % 2 == 0 { num *= 2 } else { num /= 2 };
        std::thread::sleep(dur);
        draw_cell(canvas, 3, 3, num)?;

        i += 1;
    }
}

fn full_area() -> mxcfb_rect {
    let middle_x = libremarkable::framebuffer::common::DISPLAYWIDTH / 2;
    let middle_y = libremarkable::framebuffer::common::DISPLAYHEIGHT / 2;

    mxcfb_rect {
        left: middle_x as u32 - CELL_SIZE * 2 - CELL_MARGIN * 4,
        top: middle_y as u32 - CELL_SIZE * 2 - CELL_MARGIN * 4,
        width: CELL_SIZE * 4 + CELL_MARGIN * 8,
        height: CELL_SIZE * 4 + CELL_MARGIN * 8,
    }
}

fn draw_cell(canvas: &mut Canvas, x: u32, y: u32, number: u32) -> Result<()> {
    let ref text = number.to_string();
    let cell_area = cell_area(x, y, false)?;

    // Clear any previous content
    canvas.fill_rect(
        Point2 {
            x: Some(cell_area.left as i32),
            y: Some(cell_area.top as i32),
        },
        Vector2 {
            x: cell_area.width,
            y: cell_area.height,
        },
        color::WHITE,
    );

    // Get estimated size
    let text_size = canvas.framebuffer_mut().draw_text(
        Point2 {
            x: 0f32,
            y: 500f32, // Whatever
        },
        text,
        100.0f32,
        color::BLACK,
        true,
    );

    // Draw centered (kinda) in cell_area
    canvas.draw_text(
        Point2 {
            x: Some((cell_area.left + (cell_area.width - text_size.width) / 2) as i32),
            y: Some((cell_area.top + (cell_area.height - text_size.height) / 2 + 60) as i32),
        },
        text,
        100.0f32,
    );

    canvas.update_partial(&cell_area);
    debug!("Cell {},{} => {}", x, y, text);
    Ok(())
}

fn cell_area(x: u32, y: u32, include_margin: bool) -> Result<mxcfb_rect> {
    ensure!(x < 4, "Only 0-3 allowed for X!");
    ensure!(y < 4, "Only 0-3 allowed for Y!");

    let full_area = full_area();
    if include_margin {
        Ok(mxcfb_rect {
            left: full_area.left + x * CELL_SIZE + x * 2 * CELL_MARGIN + CELL_MARGIN,
            top: full_area.top + y * CELL_SIZE + y * 2 * CELL_MARGIN + CELL_MARGIN,
            width: CELL_SIZE + CELL_MARGIN,
            height: CELL_SIZE + CELL_MARGIN,
        })
    } else {
        Ok(mxcfb_rect {
            left: full_area.left + x * CELL_SIZE + x * 2 * CELL_MARGIN,
            top: full_area.top + y * CELL_SIZE + y * 2 * CELL_MARGIN,
            width: CELL_SIZE,
            height: CELL_SIZE,
        })
    }
}

fn draw_background(canvas: &mut Canvas) {
    // Title
    let update_area = canvas.draw_text(
        Point2 {
            x: None,
            y: Some(175),
        },
        "inky-2048",
        125.0f32,
    );
    canvas.update_partial(&update_area);

    // Field background
    const OUTER_MARGIN: u32 = 4;
    let full_area = full_area();
    let update_area = canvas.draw_rect(
        Point2 {
            x: Some(full_area.left as i32 - OUTER_MARGIN as i32),
            y: Some(full_area.top as i32 - OUTER_MARGIN as i32),
        },
        Vector2 {
            x: full_area.width + OUTER_MARGIN,
            y: full_area.height + OUTER_MARGIN,
        },
        3,
    );

    for i in 0..3 {
        let vertical_x = update_area.left + (CELL_SIZE + CELL_MARGIN * 2) * (i + 1) - 1;
        canvas.framebuffer_mut().draw_line(
            Point2 {
                x: vertical_x,
                y: update_area.top,
            }
            .cast()
            .unwrap(),
            Point2 {
                x: vertical_x,
                y: update_area.top + update_area.height,
            }
            .cast()
            .unwrap(),
            3,
            color::BLACK,
        );
        let horizontal_y = update_area.top + (CELL_SIZE + CELL_MARGIN * 2) * (i + 1) - 1;
        canvas.framebuffer_mut().draw_line(
            Point2 {
                x: update_area.left,
                y: horizontal_y,
            }
            .cast()
            .unwrap(),
            Point2 {
                x: update_area.left + update_area.width,
                y: horizontal_y,
            }
            .cast()
            .unwrap(),
            3,
            color::BLACK,
        );
    }

    canvas.update_partial(&update_area);
    debug!("Background drawn.");
}
