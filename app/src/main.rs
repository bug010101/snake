use macroquad::prelude::*;
use macroquad::input::MouseButton;
use macroquad::text::measure_text;
use macroquad::time::get_time;
use snake_core::{Direction, Game, GRID};

/// 用当前时间做随机种子（wasm 下也可用，不依赖 std::time）。
fn seed() -> u64 {
    (get_time() * 1000.0) as u64
}

/// 键盘输入：方向键 / WASD。
fn handle_keys(g: &mut Game) {
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
        g.set_direction(Direction::Up);
    } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
        g.set_direction(Direction::Down);
    } else if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
        g.set_direction(Direction::Left);
    } else if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
        g.set_direction(Direction::Right);
    }
}

fn render(g: &Game) {
    clear_background(Color::new(0.10, 0.10, 0.18, 1.0));

    let sw = screen_width();
    let sh = screen_height();
    let margin = 24.0;
    let board = (sw.min(sh) - margin * 2.0).max(120.0);
    let bx = (sw - board) / 2.0;
    let by = (sh - board) / 2.0 + 14.0;
    let cell = board / GRID as f32;

    // 棋盘底
    draw_rectangle(bx, by, board, board, Color::new(0.16, 0.16, 0.27, 1.0));

    // 食物
    if g.food.x >= 0 {
        let fx = bx + g.food.x as f32 * cell;
        let fy = by + g.food.y as f32 * cell;
        let p = cell * 0.14;
        draw_rectangle(
            fx + p,
            fy + p,
            cell - p * 2.0,
            cell - p * 2.0,
            Color::new(0.96, 0.35, 0.18, 1.0),
        );
    }

    // 蛇身（头更亮）
    for (i, c) in g.snake.iter().enumerate() {
        let x = bx + c.x as f32 * cell;
        let y = by + c.y as f32 * cell;
        let p = cell * 0.08;
        let col = if i == 0 {
            Color::new(0.55, 0.95, 0.20, 1.0)
        } else {
            Color::new(0.30, 0.70, 0.22, 1.0)
        };
        draw_rectangle(x + p, y + p, cell - p * 2.0, cell - p * 2.0, col);
    }

    // 分数
    let score_text = format!("Score: {}", g.score);
    draw_text(&score_text, bx, by - 12.0, cell * 0.9, WHITE);

    // 操作提示
    let hint = "Arrow / WASD / Swipe";
    let hdim = measure_text(hint, None, 18, 1.0);
    draw_text(
        hint,
        sw / 2.0 - hdim.width / 2.0,
        by + board + 22.0,
        18.0,
        Color::new(0.7, 0.7, 0.8, 1.0),
    );

    // 结束遮罩
    if !g.alive {
        draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.62));
        let title = if g.won { "YOU WIN!" } else { "GAME OVER" };
        let ts = (sh * 0.08).clamp(28.0, 64.0);
        let tdim = measure_text(title, None, ts as u16, 1.0);
        draw_text(
            title,
            sw / 2.0 - tdim.width / 2.0,
            sh / 2.0 - 6.0,
            ts,
            Color::new(1.0, 0.45, 0.35, 1.0),
        );
        let restart = "Tap or Space to restart";
        let rdim = measure_text(restart, None, 22, 1.0);
        draw_text(
            restart,
            sw / 2.0 - rdim.width / 2.0,
            sh / 2.0 + 34.0,
            22.0,
            WHITE,
        );
    }
}

#[macroquad::main("Snake")]
async fn main() {
    let mut game = Game::new(seed());
    let mut acc: f32 = 0.0;
    let tick = 0.12_f32; // 每格步长（秒），约 8 步/秒
    let mut swipe: Option<(f32, f32)> = None;

    loop {
        if game.alive {
            handle_keys(&mut game);

            // 触屏/鼠标滑动 = 转向（手机天然支持，桌面点击拖拽也行）
            if is_mouse_button_pressed(MouseButton::Left) {
                swipe = Some(mouse_position());
            }
            if is_mouse_button_released(MouseButton::Left) {
                if let Some((sx, sy)) = swipe.take() {
                    let (ex, ey) = mouse_position();
                    let (dx, dy) = (ex - sx, ey - sy);
                    if dx.abs() > dy.abs() {
                        if dx.abs() > 20.0 {
                            game.set_direction(if dx > 0.0 {
                                Direction::Right
                            } else {
                                Direction::Left
                            });
                        }
                    } else if dy.abs() > 20.0 {
                        game.set_direction(if dy > 0.0 {
                            Direction::Down
                        } else {
                            Direction::Up
                        });
                    }
                }
            }

            // 固定步长推进
            acc += get_frame_time();
            while acc >= tick {
                game.step();
                acc -= tick;
                if !game.alive {
                    acc = 0.0;
                    swipe = None;
                    break;
                }
            }
        } else {
            swipe = None;
            if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left) {
                game = Game::new(seed());
                acc = 0.0;
            }
        }

        render(&game);
        next_frame().await;
    }
}
