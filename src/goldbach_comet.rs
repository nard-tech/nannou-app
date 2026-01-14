use nannou::prelude::*;
use std::f32::consts::FRAC_PI_2;

// Goldbach Comet
// 偶数 n ごとの素数分割数 g(n) を散布図として描画する
// n を横軸、g(n) を縦軸にした静止画を 1 フレームで描く
const START: u32 = 4;
const MAX: u32 = 10_000; // 計算する偶数の上限
const STEP: u32 = 2; // 偶数を 2 刻みで走査

// 描画設定
const WINDOW_WIDTH: u32 = 1400;
const WINDOW_HEIGHT: u32 = 700;

// 余白（軸とラベルを置くスペース）
const PADDING_LEFT: f32 = 70.0;
const PADDING_RIGHT: f32 = 30.0;
const PADDING_BOTTOM: f32 = 60.0;
const PADDING_TOP: f32 = 50.0;

const POINT_SIZE: f32 = 2.0; // プロットする四角点のサイズ
const GRID_ALPHA: f32 = 0.18; // グリッド線の透明度（0.0〜1.0）
const SHOW_GRID: bool = true; // グリッド表示の ON/OFF

const LABEL_STEP: u32 = MAX / 5; // X 軸ラベル間隔

pub fn run() {
    // nannou エントリポイント
    nannou::app(model).update(update).run();
}

// 描画に必要なデータを保持する
// points は (n, g(n)) の点群
struct Model {
    points: Vec<(f32, f32)>, // (even_n, g(n))
    max_count: u32,          // g(n) の最大値（Y スケール計算用）
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Goldbach Comet")
        .view(view)
        .build()
        .unwrap();

    // 素数表（エラトステネス）を先に用意
    let is_prime = sieve(MAX);

    // 初期化時に全点を計算・キャッシュしておき、
    // 描画側を軽くする
    let mut points = Vec::new();
    let mut max_count = 0u32;

    for n in (START..=MAX).step_by(STEP as usize) {
        let c = goldbach_pairs_count(n, &is_prime);
        max_count = max_count.max(c);
        points.push((n as f32, c as f32));
    }

    Model { points, max_count }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // 動的更新は不要
    // 静止画なので何もしない
}

/// エラトステネスの篩: 0..=limit の素数フラグを返す
fn sieve(limit: u32) -> Vec<bool> {
    let n = limit as usize;
    let mut is_prime = vec![true; n + 1];

    is_prime[0] = false;
    if n >= 1 {
        is_prime[1] = false;
    }

    let mut p = 2_usize;
    while p * p <= n {
        if is_prime[p] {
            let mut k = p * p;
            while k <= n {
                is_prime[k] = false;
                k += p;
            }
        }
        p += 1;
    }
    is_prime
}

/// 偶数 n のゴールドバッハ分割数 g(n)
/// p + q = n（p, q は素数、p<=q）を数える（順序は数えない）
fn goldbach_pairs_count(n: u32, is_prime: &[bool]) -> u32 {
    let half = n / 2; // p<=q を満たすため n/2 まで探索
    let mut count = 0u32;
    for p in 2..=half {
        if is_prime[p as usize] {
            let q = n - p; // q>=p が保証される
            if is_prime[q as usize] {
                count += 1;
            }
        }
    }
    count
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let wr = app.window_rect();

    // プロット領域（ウィンドウから余白を除いた内側）
    let left = wr.left() + PADDING_LEFT;
    let right = wr.right() - PADDING_RIGHT;
    let bottom = wr.bottom() + PADDING_BOTTOM;
    let top = wr.top() - PADDING_TOP;

    // データ範囲（ワールド座標）
    let x_min = START as f32;
    let x_max = MAX as f32;
    let y_min = 0.0f32;
    let raw_y_max = (model.max_count.max(1)) as f32;
    let desired_y_ticks = 5;
    let y_step = nice_tick_step(raw_y_max - y_min, desired_y_ticks);
    let y_max = (raw_y_max / y_step).ceil() * y_step;
    let y_tick_count = ((y_max - y_min) / y_step).round().max(1.0) as u32;

    // 軸・ラベル描画
    draw.line()
        .start(pt2(left, bottom))
        .end(pt2(right, bottom))
        .weight(1.0)
        .color(WHITE);

    draw.line()
        .start(pt2(left, bottom))
        .end(pt2(left, top))
        .weight(1.0)
        .color(WHITE);

    draw.text("Goldbach Comet")
        .x_y((left + right) * 0.5, wr.top() - 20.0)
        .color(WHITE)
        .font_size(28);

    draw.text("Even Number (n)")
        .x_y((left + right) * 0.5, wr.bottom() + 25.0)
        .color(WHITE)
        .font_size(16);

    draw.text("Number of Prime Pairs g(n)")
        .x_y(wr.left() + 18.0, (bottom + top) * 0.5)
        .rotate(FRAC_PI_2)
        .color(WHITE)
        .font_size(16);

    // グリッドと目盛り
    if SHOW_GRID {
        draw_grid(&draw, left, right, bottom, top, 10, y_tick_count as usize);
    }
    draw_ticks(
        &draw,
        left,
        right,
        bottom,
        top,
        x_min,
        x_max,
        y_min,
        y_max,
        y_step,
        y_tick_count,
    );

    // 点群をワールド座標 -> 画面座標へマッピングして描画
    for &(x, y) in &model.points {
        let px = map_range(x, x_min, x_max, left, right);
        let py = map_range(y, y_min, y_max, bottom, top);

        draw.rect()
            .x_y(px, py)
            .w_h(POINT_SIZE, POINT_SIZE)
            .color(WHITE);
    }

    draw.to_frame(app, &frame).unwrap();
}

// 描画領域を等分して補助線を引く
fn draw_grid(
    draw: &Draw,
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    x_div: usize,
    y_div: usize,
) {
    let grid_col = srgba(1.0, 1.0, 1.0, GRID_ALPHA);

    for i in 1..x_div {
        let t = i as f32 / x_div as f32;
        let x = lerp(left, right, t);
        draw.line()
            .start(pt2(x, bottom))
            .end(pt2(x, top))
            .weight(1.0)
            .color(grid_col);
    }
    for j in 1..y_div {
        let t = j as f32 / y_div as f32;
        let y = lerp(bottom, top, t);
        draw.line()
            .start(pt2(left, y))
            .end(pt2(right, y))
            .weight(1.0)
            .color(grid_col);
    }
}

fn draw_ticks(
    draw: &Draw,
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    y_step: f32,
    y_ticks: u32,
) {
    let x_min_u = x_min.ceil().max(0.0) as u32;
    let x_max_u = x_max.floor().max(0.0) as u32;

    // X 軸は LABEL_STEP ごとにラベルを打つ
    let mut v = x_min_u.div_ceil(LABEL_STEP) * LABEL_STEP;

    while v <= x_max_u {
        let px = map_range(v as f32, x_min, x_max, left, right);

        // tick
        draw.line()
            .start(pt2(px, bottom))
            .end(pt2(px, bottom - 6.0))
            .weight(1.0)
            .color(WHITE);

        draw.text(&format!("{}", v))
            .x_y(px, bottom - 18.0)
            .color(WHITE)
            .font_size(12);

        v += LABEL_STEP;
    }

    // Y 軸は切り上げた上限とキリの良い間隔で配置
    for i in 0..=y_ticks {
        let value = y_min + y_step * i as f32;
        let py = map_range(value, y_min, y_max, bottom, top);
        let label = if y_step.fract().abs() < f32::EPSILON {
            format!("{:.0}", value)
        } else {
            format!("{:.2}", value)
        };

        draw.line()
            .start(pt2(left, py))
            .end(pt2(left - 6.0, py))
            .weight(1.0)
            .color(WHITE);

        draw.text(&label)
            .x_y(left - 24.0, py)
            .color(WHITE)
            .font_size(12);
    }
}

fn nice_tick_step(range: f32, desired_ticks: u32) -> f32 {
    let desired = desired_ticks.max(1) as f32;
    let step = nice_number(range / desired, true);
    if step > 0.0 {
        step
    } else {
        1.0
    }
}

// グラフの軸に使う「キリの良い」数値を返す（1, 2, 5 系列）
fn nice_number(x: f32, round: bool) -> f32 {
    if x == 0.0 {
        return 0.0;
    }

    let exp = x.abs().log10().floor();
    let f = x / 10f32.powf(exp);
    let nf = if round {
        if f < 1.5 {
            1.0
        } else if f < 3.0 {
            2.0
        } else if f < 7.0 {
            5.0
        } else {
            10.0
        }
    } else if f <= 1.0 {
        1.0
    } else if f <= 2.0 {
        2.0
    } else if f <= 5.0 {
        5.0
    } else {
        10.0
    };

    nf * 10f32.powf(exp)
}

// 線形補間
// グリッドや目盛りの位置計算に利用
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
