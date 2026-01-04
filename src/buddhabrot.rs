use nannou::prelude::*;
use nannou::rand;
use nannou::wgpu;

// 画面サイズ / Image resolution
const WINDOW_WIDTH: u32 = 1000;
const WINDOW_HEIGHT: u32 = 1000;

// Buddhabrot 計算パラメータ / Computation parameters
// 1サンプル（複素数c）につき、最大何回反復するか / Maximum iteration count per sample (complex parameter c).
const MAX_ITER: u32 = 10_000;

// 1フレームでランダムに試す c の個数（多いほど早く濃くなるが重くなる）
// Number of random c samples per frame. More samples = faster convergence but heavier CPU load.
const SAMPLES_PER_FRAME: usize = 20_000;

// 複素平面のサンプリング範囲（Mandelbrot の定番領域）
// Common Mandelbrot viewing region (we sample c from here).
const RE_MIN: f64 = -2.0;
const RE_MAX: f64 = 1.0;
const IM_MIN: f64 = -1.5;
const IM_MAX: f64 = 1.5;

// エントリポイント / Entry point
// nannou アプリを起動する / Launch the nannou app.
pub fn run() {
    nannou::app(model).update(update).run();
}

// モデル / Model
// 計算結果（counts）を保持し、RGBAへ変換してテクスチャへアップロードして表示する
// Holds accumulation buffers (counts), converts them into RGBA, uploads to GPU texture, and displays it.
struct Model {
    texture: wgpu::Texture,

    // 各ピクセルのヒット回数（軌道が通った回数）を蓄積する
    // Per-pixel hit counts (how many orbit points landed on each pixel).
    counts: Vec<u32>,

    // 表示用のRGBAバッファ（countsをトーンマップして作る）
    // RGBA buffer to display (tone-mapped from counts).
    rgba: Vec<u8>,

    // 正規化用に、counts の最大値を追跡しておく
    // Track max count for normalization.
    max_count: u32,

    // rgba を更新したかどうか（本来は view で false に戻したい）
    // Whether RGBA has been updated (ideally set back to false after upload).
    dirty: bool,
}

// 初期化 / Initialization
fn model(app: &App) -> Model {
    // ウィンドウ生成（view 関数で描画する）
    // Create a window; rendering is done in `view`.
    app.new_window()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .view(view)
        .build()
        .unwrap();

    let window = app.main_window();

    // 空のテクスチャをGPU側に作っておく。
    // 毎フレーム、CPU側で作った rgba を upload_data でアップロードする。
    // Create an empty GPU texture. Each frame we upload CPU-generated RGBA via `upload_data`.
    let texture = wgpu::TextureBuilder::new()
        .size([WINDOW_WIDTH, WINDOW_HEIGHT])
        .format(wgpu::TextureFormat::Rgba8UnormSrgb)
        .usage(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING)
        .build(window.device());

    // counts は 1ピクセルにつき 1要素、RGBA は 1ピクセルにつき 4要素
    // `counts` has one u32 per pixel; `rgba` has 4 u8 per pixel.
    let counts = vec![0u32; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize];
    let rgba = vec![0u8; (WINDOW_WIDTH * WINDOW_HEIGHT * 4) as usize];

    Model {
        texture,
        counts,
        rgba,
        max_count: 1,
        dirty: true,
    }
}

// 更新（計算） / Update (CPU computation)
// 毎フレーム、複素数 c をランダムに多数サンプリングし、
// 発散したものだけ軌道(zの列)をcountsに加算していく（Buddhabrot）。
// Each frame, sample many random complex parameters c.
// For escaping ones only, accumulate their orbit points into counts (Buddhabrot).
fn update(_app: &App, model: &mut Model, _update: Update) {
    // 軌道（zの履歴）を入れるバッファ。毎サンプルで再利用してアロケを避ける。
    // Orbit buffer reused per sample to avoid repeated allocations.
    let mut orbit: Vec<(f64, f64)> = Vec::with_capacity(1024);

    // 1フレームで SAMPLES_PER_FRAME 回だけ c を試す
    // Try SAMPLES_PER_FRAME random c values per frame.
    for _ in 0..SAMPLES_PER_FRAME {
        // c を複素平面から一様にランダムサンプル
        // Uniformly sample c from the complex plane region.
        let cr: f64 = rand::random_range(RE_MIN, RE_MAX);
        let ci: f64 = rand::random_range(IM_MIN, IM_MAX);

        // 前回の軌道履歴をクリアして再利用
        // Clear and reuse the orbit vector.
        orbit.clear();

        // z0 = 0 から始める（Mandelbrot反復）
        // Start iteration from z0 = 0 (Mandelbrot iteration).
        let mut zr = 0.0f64;
        let mut zi = 0.0f64;

        // 発散したかどうか（Buddhabrotでは“発散した点”の軌道だけを使う）
        // Whether the orbit escaped (Buddhabrot uses orbits of escaping points).
        let mut escaped = false;

        // z_{n+1} = z_n^2 + c を反復
        // Iterate z_{n+1} = z_n^2 + c.
        for _ in 0..MAX_ITER {
            // (zr + i*zi)^2 + (cr + i*ci)
            let zr2 = zr * zr - zi * zi + cr;
            let zi2 = 2.0 * zr * zi + ci;
            zr = zr2;
            zi = zi2;

            // |z|^2 > 4 なら発散とみなす（脱出半径2）
            // Escape test: if |z|^2 > 4, the orbit escapes (escape radius 2).
            if zr * zr + zi * zi > 4.0 {
                escaped = true;
                break;
            }

            // 非発散のステップは軌道として蓄積（後で投影してcountsに加算）
            // Store non-escaped steps into orbit for later projection.
            // TODO: `if zr * zr + zi * zi > 4.0` の前に入れるべきか？
            orbit.push((zr, zi));
        }

        // 発散した場合のみ、軌道点を2D画像座標に投影して counts を加算
        // If escaped, project orbit points onto the image and increment counts.
        if escaped {
            for &(orbit_re, orbit_im) in &orbit {
                // 複素平面座標 -> ピクセル座標へ線形変換
                // Linear mapping from complex plane coordinates to pixel coordinates.
                let x = ((orbit_re - RE_MIN) / (RE_MAX - RE_MIN) * (WINDOW_WIDTH as f64)) as i32;
                let y = ((orbit_im - IM_MIN) / (IM_MAX - IM_MIN) * (WINDOW_HEIGHT as f64)) as i32;
                // 範囲内なら counts に加算
                // If within bounds, increment the hit count.
                if (0..WINDOW_WIDTH as i32).contains(&x) && (0..WINDOW_HEIGHT as i32).contains(&y) {
                    let idx = (y as u32 * WINDOW_WIDTH + x as u32) as usize;

                    // saturating_add によりオーバーフローを防ぐ
                    // Use saturating_add to prevent overflow.
                    let v = model.counts[idx].saturating_add(1);
                    model.counts[idx] = v;

                    // 最大値を更新（正規化に使う）
                    // Track max value for normalization.
                    if v > model.max_count {
                        model.max_count = v;
                    }
                }
            }
        }
    }

    // counts -> rgba（可視化） / Visualization (tone mapping)
    // counts のダイナミックレンジが非常に広いので、logスケールにして見えるようにする
    // counts have a huge dynamic range; use logarithmic scaling for visibility.
    let max_c = model.max_count.max(1) as f64;
    let denom = (max_c + 1.0).ln();

    for i in 0..(WINDOW_WIDTH * WINDOW_HEIGHT) as usize {
        let c = model.counts[i] as f64;

        // log1p(count) / log1p(max) を 0..255 にスケール
        // Scale log1p(count)/log1p(max) into 0..255.
        let t = ((c + 1.0).ln() / denom * 255.0).clamp(0.0, 255.0) as u8;

        // ここではグレースケール（R=G=B=t）
        // Grayscale (R=G=B=t).
        let o = i * 4;
        model.rgba[o] = t;
        model.rgba[o + 1] = t;
        model.rgba[o + 2] = t;
        model.rgba[o + 3] = 255;
    }

    // rgba 更新済みフラグ
    // Mark RGBA as updated.
    model.dirty = true;
}

// 描画 / Render
// frame に対してテクスチャを描画する。
// dirty の場合、CPU側 rgba を GPU テクスチャへ upload_data で転送してから描く。
// Draw the texture to the frame.
// If dirty, upload CPU RGBA to the GPU texture via upload_data before drawing.
fn view(app: &App, model: &Model, frame: Frame) {
    // 背景を黒でクリア
    // Clear background to black.
    frame.clear(BLACK);

    // rgba を GPU テクスチャへ反映（upload_data を使う）
    // Upload RGBA to the GPU texture using upload_data.
    if model.dirty {
        // コマンドエンコーダを取得し、そこにアップロード命令を積む
        // Get a command encoder and record the upload commands.
        let mut encoder = frame.command_encoder();

        // device を取得（frame が持つ device_queue_pair から）
        // Obtain the device from the frame's device_queue_pair.
        let device = frame.device_queue_pair().device();

        // CPUの rgba バッファを GPU テクスチャにコピー
        // Copy CPU RGBA buffer into the GPU texture.
        model.texture.upload_data(device, &mut encoder, &model.rgba);

        // NOTE:
        // view は &Model なので dirty を false に戻せない。
        // もし「更新があるときだけupload」したいなら、
        // - raw_view を使う（&mut Model が取れるようにする）
        // - もしくは update 側で “次フレームでupload済み扱い” にする
        //
        // view takes &Model so we cannot set dirty=false here.
        // If you want upload-only-when-needed, consider using raw_view
        // or manage the flag on update side.
    }

    // テクスチャをウィンドウ全体に貼り付けて描画
    // Draw the texture stretched to the window rectangle.
    let draw = app.draw();
    draw.texture(&model.texture).wh(app.window_rect().wh());
    draw.to_frame(app, &frame).unwrap();
}
