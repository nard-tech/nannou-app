use crate::mathlib::prime::Prime;
use nannou::prelude::*;

// ウラムの螺旋を静的に描画するための設定値 / Configuration for rendering a static Ulam spiral.
const WINDOW_SIZE: u32 = 1200;
const SPIRAL_SIDE_LEN: i32 = 201;
const PRIME_MARKER_SIZE: f32 = 4.0;
const PRIME_COLOR: Srgb<u8> = WHITE;

// nannou アプリのエントリポイント / Entry point for the nannou app.
pub fn run() {
    nannou::app(model).update(update).run();
}

// 描画に必要な前計算済みデータを保持する / Holds precomputed data needed for rendering.
struct Model {
    // 素数に対応する螺旋上の描画座標 / Draw positions on the spiral for prime numbers.
    prime_positions: Vec<Point2>,
    // 補助軸を引くための半径 / Half span used to draw the guide axes.
    axis_half_span: f32,
    // 素数マーカーの一辺 / Side length of each prime marker.
    point_size: f32,
}

// ウラムの螺旋を 1 マスずつ前進するための状態 / State for walking the Ulam spiral one cell at a time.
struct SpiralWalker {
    // 現在の格子座標 / Current grid coordinate.
    x: i32,
    y: i32,
    // 現在進んでいる方向ベクトル / Direction vector for the current segment.
    dx: i32,
    dy: i32,
    // 同じ歩数で進む区間の長さ / Number of steps to take in the current segment.
    segment_len: i32,
    // 現区間で何歩進んだか / Steps already taken in the current segment.
    steps_in_segment: i32,
    // 現在の長さで何区間こなしたか / Number of completed segments at the current length.
    segments_at_len: i32,
}

// 初期化時に素数表と螺旋座標をまとめて計算する / Precompute the prime table and spiral positions at startup.
fn model(app: &App) -> Model {
    app.new_window()
        .size(WINDOW_SIZE, WINDOW_SIZE)
        .title("Ulam Spiral")
        .view(view)
        .build()
        .unwrap();

    let max_value = spiral_max_value(SPIRAL_SIDE_LEN);
    let is_prime = Prime::sieve(max_value);
    let spacing = point_spacing(app.window_rect(), SPIRAL_SIDE_LEN);
    let half_extent = (SPIRAL_SIDE_LEN as f32 - 1.0) * 0.5 * spacing;

    // 螺旋上の整数を走査し、素数だけを描画点として保持する / Walk the spiral integers and keep only primes as drawable points.
    let mut prime_positions = Vec::new();
    let mut walker = SpiralWalker::new();
    for value in 1..=max_value {
        let (gx, gy) = walker.position();
        if is_prime[value as usize] {
            prime_positions.push(pt2(gx as f32 * spacing, gy as f32 * spacing));
        }

        walker.advance();
    }

    Model {
        prime_positions,
        axis_half_span: half_extent,
        point_size: PRIME_MARKER_SIZE.min(spacing * 0.7).max(1.0),
    }
}

// この描画は静的なのでフレーム更新では何もしない / The drawing is static, so no per-frame update is required.
fn update(_app: &App, _model: &mut Model, _update: Update) {}

// 軸と素数マーカーを描画する / Render the guide axes and prime markers.
fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    draw.line()
        .start(pt2(-model.axis_half_span, 0.0))
        .end(pt2(model.axis_half_span, 0.0))
        .weight(1.0)
        .color(axis_color());

    draw.line()
        .start(pt2(0.0, -model.axis_half_span))
        .end(pt2(0.0, model.axis_half_span))
        .weight(1.0)
        .color(axis_color());

    for &position in &model.prime_positions {
        draw.rect()
            .xy(position)
            // 小さな正方形で素数を打つ / Plot each prime as a small square marker.
            .w_h(model.point_size, model.point_size)
            .color(PRIME_COLOR);
    }

    draw.to_frame(app, &frame).unwrap();
}

// 正方形の一辺から、螺旋に含める最大整数を求める / Convert the spiral side length into the maximum integer contained in the square.
fn spiral_max_value(side_len: i32) -> u32 {
    assert!(side_len > 0, "side_len must be positive");
    assert!(side_len % 2 == 1, "side_len must be odd");

    let side_len = side_len as u32;
    side_len
        .checked_mul(side_len)
        .expect("spiral_max_value: side_len is too large to square without overflowing u32")
}

// ウィンドウ内に螺旋全体が収まるよう点間隔を決める / Compute the point spacing so the whole spiral fits inside the window.
fn point_spacing(window_rect: Rect, side_len: i32) -> f32 {
    let usable = window_rect.w().min(window_rect.h()) * 0.9;
    usable / side_len as f32
}

// 背景より少し薄い補助軸の色 / A subtle axis color that stays dimmer than the prime markers.
fn axis_color() -> Srgba<u8> {
    srgba(255, 255, 255, 38)
}

impl SpiralWalker {
    // 原点から始まるウラムの螺旋ウォーカーを作る / Create an Ulam spiral walker starting at the origin.
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            dx: 1,
            dy: 0,
            segment_len: 1,
            steps_in_segment: 0,
            segments_at_len: 0,
        }
    }

    // 現在位置を返す / Return the current position.
    fn position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    // 次の整数に対応する位置へ進む / Advance to the position for the next integer.
    fn advance(&mut self) {
        self.x += self.dx;
        self.y += self.dy;
        self.steps_in_segment += 1;

        if self.steps_in_segment == self.segment_len {
            self.steps_in_segment = 0;

            // 90度反時計回りに向きを変える / Rotate the direction 90 degrees counterclockwise.
            let old_dx = self.dx;
            self.dx = -self.dy;
            self.dy = old_dx;
            self.segments_at_len += 1;

            // 同じ長さの区間を 2 本進んだら歩数を増やす / Increase the segment length after completing two segments of the same length.
            if self.segments_at_len == 2 {
                self.segments_at_len = 0;
                self.segment_len += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{spiral_max_value, SpiralWalker};

    #[test]
    // 一辺 N の正方形に N^2 個の整数が入ることを確認する / Confirm that a square of side N contains N^2 integers.
    fn spiral_max_value_matches_square_side() {
        assert_eq!(spiral_max_value(1), 1);
        assert_eq!(spiral_max_value(5), 25);
        assert_eq!(spiral_max_value(201), 40_401);
    }

    #[test]
    // u32 に収まらない正方形サイズは明示的に panic することを確認する / Confirm that an oversized square side panics instead of silently overflowing u32.
    #[should_panic(
        expected = "spiral_max_value: side_len is too large to square without overflowing u32"
    )]
    fn spiral_max_value_panics_on_overflow() {
        spiral_max_value(65_537);
    }

    #[test]
    // ウォーカーが標準的なウラムの螺旋の並びで進むことを確認する / Confirm that the walker follows the canonical Ulam spiral ordering.
    fn spiral_walker_follows_ulam_spiral_order() {
        let expected = [
            (1, (0, 0)),
            (2, (1, 0)),
            (3, (1, 1)),
            (4, (0, 1)),
            (5, (-1, 1)),
            (6, (-1, 0)),
            (7, (-1, -1)),
            (8, (0, -1)),
            (9, (1, -1)),
            (10, (2, -1)),
            (11, (2, 0)),
            (12, (2, 1)),
            (13, (2, 2)),
            (14, (1, 2)),
            (15, (0, 2)),
            (16, (-1, 2)),
            (17, (-2, 2)),
        ];

        let mut walker = SpiralWalker::new();
        for (value, coord) in expected {
            assert_eq!(walker.position(), coord, "value={value}");
            walker.advance();
        }
    }
}
