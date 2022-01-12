mod canvas;
mod swipe;

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

use anyhow::Result;
use canvas::{color, mxcfb_rect, Canvas, Point2, Vector2};
use fxhash::FxHashMap;
use libremarkable::framebuffer::FramebufferDraw;
use libremarkable::image::{DynamicImage, RgbImage, Rgba, RgbaImage};
use libremarkable::input::{ev::EvDevContext, InputDevice, InputEvent};
use once_cell::sync::Lazy;
use play_2048::{
    board::{Board, Direction},
    game::*,
};
use std::time::{Duration, Instant};
use std::{env, thread};
use swipe::{Direction as SwipeDirection, Swipe, SwipeTracker, Trigger};

#[rustfmt::skip]
static NUMBER_IMAGES: Lazy<FxHashMap<u16, RgbaImage>> = Lazy::new(|| {
    let mut map = FxHashMap::default();
    map.insert(2, libremarkable::image::load_from_memory(include_bytes!("../res/2.png")).unwrap().to_rgba());
    map.insert(4, libremarkable::image::load_from_memory(include_bytes!("../res/4.png")).unwrap().to_rgba());
    map.insert(8, libremarkable::image::load_from_memory(include_bytes!("../res/8.png")).unwrap().to_rgba());
    map.insert(16, libremarkable::image::load_from_memory(include_bytes!("../res/16.png")).unwrap().to_rgba());
    map.insert(32, libremarkable::image::load_from_memory(include_bytes!("../res/32.png")).unwrap().to_rgba());
    map.insert(64, libremarkable::image::load_from_memory(include_bytes!("../res/64.png")).unwrap().to_rgba());
    map.insert(128, libremarkable::image::load_from_memory(include_bytes!("../res/128.png")).unwrap().to_rgba());
    map.insert(256, libremarkable::image::load_from_memory(include_bytes!("../res/256.png")).unwrap().to_rgba());
    map.insert(512, libremarkable::image::load_from_memory(include_bytes!("../res/512.png")).unwrap().to_rgba());
    map.insert(1024, libremarkable::image::load_from_memory(include_bytes!("../res/1024.png")).unwrap().to_rgba());
    map.insert(2048, libremarkable::image::load_from_memory(include_bytes!("../res/2048.png")).unwrap().to_rgba());
    map.insert(4096, libremarkable::image::load_from_memory(include_bytes!("../res/4096.png")).unwrap().to_rgba());
    map.insert(8192, libremarkable::image::load_from_memory(include_bytes!("../res/8192.png")).unwrap().to_rgba());
    map.insert(16384, libremarkable::image::load_from_memory(include_bytes!("../res/16384.png")).unwrap().to_rgba());
    map
});

#[rustfmt::skip]
static BG_IMAGES: Lazy<FxHashMap<u16, RgbImage>> = Lazy::new(|| {
    let mut map = FxHashMap::default();
    map.insert(2, libremarkable::image::load_from_memory(include_bytes!("../res/bg_2.png")).unwrap().to_rgb());
    map.insert(4, libremarkable::image::load_from_memory(include_bytes!("../res/bg_4.png")).unwrap().to_rgb());
    map.insert(8, libremarkable::image::load_from_memory(include_bytes!("../res/bg_8.png")).unwrap().to_rgb());
    map.insert(16, libremarkable::image::load_from_memory(include_bytes!("../res/bg_16.png")).unwrap().to_rgb());
    map.insert(32, libremarkable::image::load_from_memory(include_bytes!("../res/bg_32.png")).unwrap().to_rgb());
    map.insert(64, libremarkable::image::load_from_memory(include_bytes!("../res/bg_64.png")).unwrap().to_rgb());
    map.insert(128, libremarkable::image::load_from_memory(include_bytes!("../res/bg_128.png")).unwrap().to_rgb());
    map.insert(256, libremarkable::image::load_from_memory(include_bytes!("../res/bg_256.png")).unwrap().to_rgb());
    map.insert(512, libremarkable::image::load_from_memory(include_bytes!("../res/bg_512.png")).unwrap().to_rgb());
    map.insert(1024, libremarkable::image::load_from_memory(include_bytes!("../res/bg_1024.png")).unwrap().to_rgb());
    map.insert(2048, libremarkable::image::load_from_memory(include_bytes!("../res/bg_2048.png")).unwrap().to_rgb());
    map.insert(4096, libremarkable::image::load_from_memory(include_bytes!("../res/bg_4096.png")).unwrap().to_rgb());
    map.insert(8192, libremarkable::image::load_from_memory(include_bytes!("../res/bg_8192.png")).unwrap().to_rgb());
    map.insert(16384, libremarkable::image::load_from_memory(include_bytes!("../res/bg_16384.png")).unwrap().to_rgb());
    map
});

fn duify_image(img: RgbaImage) -> RgbaImage {
    RgbaImage::from_fn(img.width(), img.height(), |x, y| {
        let [r, g, b, a] = img.get_pixel(x, y).data;
        let decided_rgb = if r as u16 + g as u16 + b as u16 > 127 * 3 {
            255
        } else {
            0
        };
        let decided_alpha = if a > 127 { 255 } else { 0 };
        Rgba([decided_rgb, decided_rgb, decided_rgb, decided_alpha])
    })
}

impl From<SwipeDirection> for Direction {
    fn from(swipe_dir: SwipeDirection) -> Self {
        match swipe_dir {
            SwipeDirection::Up => Direction::Up,
            SwipeDirection::Right => Direction::Right,
            SwipeDirection::Down => Direction::Down,
            SwipeDirection::Left => Direction::Left,
        }
    }
}

const CELL_SIZE: u32 = 320;
const CELL_MARGIN: u32 = 4;

fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "INFO");
    }
    env_logger::builder().format_timestamp_millis().init();

    let mut canvas = Canvas::new();
    canvas.clear();
    canvas.update_full();

    draw_background(&mut canvas);

    // Initialize game
    let mut game = GameBuilder::default().build();
    let mut board = game.board;
    draw_changed_cells(&mut canvas, None, board)?;
    //game.board.move_to(direction)

    // Input loop
    let (input_tx, input_rx) = std::sync::mpsc::channel();
    EvDevContext::new(InputDevice::Multitouch, input_tx).start();
    let mut swipe_tracker = SwipeTracker::new();
    let swipes = &[
        Swipe {
            direction: SwipeDirection::Up,
            trigger: Trigger::Completed,
        },
        Swipe {
            direction: SwipeDirection::Right,
            trigger: Trigger::Completed,
        },
        Swipe {
            direction: SwipeDirection::Down,
            trigger: Trigger::Completed,
        },
        Swipe {
            direction: SwipeDirection::Left,
            trigger: Trigger::Completed,
        },
    ];

    for event in input_rx {
        match event {
            InputEvent::MultitouchEvent { event } => {
                for swipe in swipe_tracker.detect(event, swipes) {
                    info!("Swiped {:?}", swipe.direction);
                    let start = Instant::now();
                    let last = board;
                    game.play(swipe.direction.into());
                    draw_changed_cells(&mut canvas, Some(last), game.board)?;
                    if last != game.board && game.board.count_empty_tiles() > 0 {
                        thread::sleep(Duration::from_millis(350).saturating_sub(start.elapsed()));
                        let last = game.board;
                        game.populate_new_tile();
                        draw_changed_cells(&mut canvas, Some(last), game.board)?;
                    }
                    board = game.board;
                }
            }
            _ => {
                bail!("Unexpected input event type!")
            }
        }
    }

    info!("Bye!");
    Ok(())
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

fn draw_changed_cells(canvas: &mut Canvas, last: Option<Board>, current: Board) -> Result<()> {
    debug!("Board: {}", current);
    let start = Instant::now();
    for i in 0..16 {
        let draw_value = {
            if let Some(last) = last {
                let last_val = last.get_value(i);
                let cur_val = current.get_value(i);
                if last_val == cur_val {
                    continue;
                } else {
                    cur_val
                }
            } else {
                current.get_value(i)
            }
        };
        draw_cell(canvas, i as u32 % 4, i as u32 / 4, draw_value)?;
    }
    debug!("Update took {:?}", start.elapsed());
    Ok(())
}

fn draw_cell(canvas: &mut Canvas, x: u32, y: u32, number: u16) -> Result<()> {
    /*let ref text = if number == 0 {
        "".to_owned()
    } else {
        number.to_string()
    };*/
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

    /*
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
    );*/

    if number > 0 {
        let bg = BG_IMAGES
            .get(&number)
            .ok_or(anyhow!("No bg image for number found!"))?;
        let (img_width, img_height) = bg.dimensions();
        let bg = DynamicImage::ImageRgb8(bg.clone());
        canvas.draw_image(
            Point2 {
                x: cell_area.left + (cell_area.width - img_width) / 2,
                y: cell_area.top + (cell_area.height - img_height) / 2,
            }
            .cast()
            .unwrap(),
            &bg,
            false,
        );

        let img = NUMBER_IMAGES
            .get(&number)
            .ok_or(anyhow!("No image for number found!"))?;
        let (img_width, img_height) = img.dimensions();
        let img = DynamicImage::ImageRgba8(img.clone());
        canvas.draw_image(
            Point2 {
                x: cell_area.left + (cell_area.width - img_width) / 2,
                y: cell_area.top + (cell_area.height - img_height) / 2,
            }
            .cast()
            .unwrap(),
            &img,
            true,
        );
    }

    canvas.update_partial(&cell_area);
    debug!("Cell {},{} => {}", x, y, number);
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
