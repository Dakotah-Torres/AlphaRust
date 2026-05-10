/// swing_bench.rs — AlphaRust Swing Detector Benchmark
/// Place at: src/bin/swing_bench.rs
///
/// Run: cargo run --bin swing_bench --release

use AlphaRust::detectors::primatives::candles::candle::Candle;
use AlphaRust::detectors::swings::swing_streaming::{
    ConfirmationType, StremingSwingDetector, SwingMode, SwingType,
};
use AlphaRust::engine::ingester::CsvIngester;

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

// ── Confirmation mode configs ─────────────────────────────────────────────────

struct ModeConfig {
    label:        &'static str,
    confirmation: fn() -> ConfirmationType,
}

fn mode_configs() -> Vec<ModeConfig> {
    vec![
        ModeConfig {
            label: "PercentCandle (0.3%)",
            confirmation: || ConfirmationType::PercentCandle(0.003),
        },
        ModeConfig {
            label: "PercentPrice (0.01%)",
            confirmation: || ConfirmationType::PercentPrice(0.0001),
        },
        ModeConfig {
            label: "Ticks (4 @ 0.25)",
            confirmation: || ConfirmationType::Ticks(4, 0.25),
        },
        ModeConfig {
            label: "Candles (1)",
            confirmation: || ConfirmationType::Candles(1),
        },
    ]
}

// ── Types ─────────────────────────────────────────────────────────────────────

struct BenchResult {
    mode_label:        String,
    candles_processed: usize,
    highs_detected:    usize,
    lows_detected:     usize,
    ingest_ms:         f64,
    detect_ms:         f64,
    total_ms:          f64,
    candles_per_sec:   f64,
}

type SwingPoint = (u64, f64, bool); // (timestamp, price, is_high)

// ── Clone ConfirmationType (doesn't derive Clone) ─────────────────────────────

fn clone_confirmation(c: &ConfirmationType) -> ConfirmationType {
    match c {
        ConfirmationType::PercentCandle(v) => ConfirmationType::PercentCandle(*v),
        ConfirmationType::PercentPrice(v)  => ConfirmationType::PercentPrice(*v),
        ConfirmationType::Ticks(t, s)      => ConfirmationType::Ticks(*t, *s),
        ConfirmationType::Candles(n)       => ConfirmationType::Candles(*n),
    }
}

// ── Run one benchmark ─────────────────────────────────────────────────────────

fn run_benchmark(
    csv_path:     &PathBuf,
    mode_label:   &str,
    confirmation: ConfirmationType,
) -> (BenchResult, Vec<Candle>, Vec<SwingPoint>) {

    let ingest_start = Instant::now();
    let file = File::open(csv_path).expect("Failed to open CSV");
    let candles: Vec<Candle> = CsvIngester::new(file).collect();
    let ingest_ms = ingest_start.elapsed().as_secs_f64() * 1000.0;
    let candle_count = candles.len();

    let conf_low = clone_confirmation(&confirmation);

    let detect_start = Instant::now();

    let mut high_det = StremingSwingDetector::new(
        SwingMode::Wick, SwingType::High, confirmation, 1000,
    );
    for &c in &candles { high_det.process_candle(c); }
    let high_swings = high_det.get_swings();

    let mut low_det = StremingSwingDetector::new(
        SwingMode::Wick, SwingType::Low, conf_low, 1000,
    );
    for &c in &candles { low_det.process_candle(c); }
    let low_swings = low_det.get_swings();

    let detect_ms = detect_start.elapsed().as_secs_f64() * 1000.0;
    let total_ms  = ingest_ms + detect_ms;

    let highs = high_swings.len();
    let lows  = low_swings.len();

    let mut all_swings: Vec<SwingPoint> = high_swings
        .into_iter().map(|(ts, p, _)| (ts, p, true))
        .chain(low_swings.into_iter().map(|(ts, p, _)| (ts, p, false)))
        .collect();
    all_swings.sort_by_key(|(ts, _, _)| *ts);

    let result = BenchResult {
        mode_label: mode_label.to_string(),
        candles_processed: candle_count,
        highs_detected: highs,
        lows_detected: lows,
        ingest_ms,
        detect_ms,
        total_ms,
        candles_per_sec: candle_count as f64 / (total_ms / 1000.0),
    };

    (result, candles, all_swings)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn format_num(n: usize) -> String {
    let s = n.to_string();
    let mut r = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { r.push(','); }
        r.push(ch);
    }
    r.chars().rev().collect()
}

fn format_rate(r: f64) -> String {
    if r >= 1_000_000.0  { format!("{:.2}M/s", r / 1_000_000.0) }
    else if r >= 1_000.0 { format!("{:.1}K/s", r / 1_000.0) }
    else                 { format!("{:.0}/s", r) }
}

// ── HTML generation ───────────────────────────────────────────────────────────

fn generate_html(
    filename:    &str,
    all_results: &[BenchResult],
    all_swings:  &[Vec<SwingPoint>],
    candles:     &[Candle],
) -> String {

    let max_pts = 800usize;
    let step    = (candles.len() / max_pts).max(1);
    let chart_c: Vec<&Candle> = candles.iter().step_by(step).collect();

    let candle_json: String = chart_c.iter().map(|c| format!(
        "{{t:{},o:{:.2},h:{:.2},l:{:.2},c:{:.2}}}",
        c.timestamp, c.open, c.high, c.low, c.close
    )).collect::<Vec<_>>().join(",");

    let swing_jsons: Vec<String> = all_swings.iter().enumerate().map(|(i, sw)| {
        let label = &all_results[i].mode_label;
        let pts: String = sw.iter().map(|(ts, p, h)| format!(
            "{{t:{},p:{:.2},h:{}}}", ts, p, if *h {"true"} else {"false"}
        )).collect::<Vec<_>>().join(",");
        format!("{{label:{:?},points:[{}]}}", label, pts)
    }).collect();
    let swing_json = format!("[{}]", swing_jsons.join(","));

    let table_rows: String = all_results.iter().map(|r| {
        let total = r.highs_detected + r.lows_detected;
        let rate  = if r.candles_processed > 0 {
            total as f64 / r.candles_processed as f64 * 100.0
        } else { 0.0 };
        format!(
            r#"<tr><td class="mode">{}</td><td>{}</td><td class="hi">{}</td><td class="lo">{}</td><td>{:.1}ms</td><td>{:.1}ms</td><td>{:.1}ms</td><td>{}</td><td>{:.2}%</td></tr>"#,
            r.mode_label,
            format_num(r.candles_processed),
            format_num(r.highs_detected),
            format_num(r.lows_detected),
            r.ingest_ms, r.detect_ms, r.total_ms,
            format_rate(r.candles_per_sec),
            rate,
        )
    }).collect();

    format!(r##"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AlphaRust — {filename}</title>
<style>
@import url('https://fonts.googleapis.com/css2?family=DM+Mono:wght@400;500&family=Syne:wght@700&display=swap');
:root{{--bg:#080810;--sur:#0f0f1a;--bor:#1a1a2e;--acc:#7c6af7;--hi:#2ecc8f;--lo:#e8404f;--txt:#dddaf0;--mut:#5a576e}}
*{{box-sizing:border-box;margin:0;padding:0}}
body{{background:var(--bg);color:var(--txt);font-family:'DM Mono',monospace;font-size:13px;padding:28px}}
header{{display:flex;align-items:baseline;gap:14px;margin-bottom:28px;border-bottom:1px solid var(--bor);padding-bottom:14px}}
header h1{{font-family:'Syne',sans-serif;font-size:20px;color:var(--acc)}}
header span{{color:var(--mut);font-size:11px}}
.card{{background:var(--sur);border:1px solid var(--bor);border-radius:8px;padding:16px;margin-bottom:20px}}
.card h2{{font-family:'Syne',sans-serif;font-size:13px;margin-bottom:12px}}
.tb{{display:flex;gap:7px;flex-wrap:wrap;margin-bottom:10px}}
.btn{{background:var(--bg);border:1px solid var(--bor);color:var(--mut);padding:3px 10px;border-radius:4px;cursor:pointer;font-family:'DM Mono',monospace;font-size:11px;transition:all .15s}}
.btn.on{{border-color:var(--acc);color:var(--acc);background:rgba(124,106,247,.08)}}
canvas{{width:100%;height:400px;display:block;cursor:crosshair}}
.legend{{display:flex;gap:14px;font-size:11px;color:var(--mut);margin-top:8px}}
.legend span{{display:flex;align-items:center;gap:5px}}
.dot{{width:8px;height:8px;border-radius:50%;display:inline-block}}
table{{width:100%;border-collapse:collapse;font-size:11px}}
thead th{{text-align:left;padding:7px 9px;color:var(--mut);border-bottom:1px solid var(--bor);white-space:nowrap}}
tbody tr{{border-bottom:1px solid var(--bor)}}
tbody tr:hover{{background:rgba(255,255,255,.02)}}
tbody tr:last-child{{border-bottom:none}}
tbody td{{padding:8px 9px;white-space:nowrap}}
.mode{{color:var(--acc)}}.hi{{color:var(--hi)}}.lo{{color:var(--lo)}}
</style></head><body>
<header><h1>AlphaRust</h1><span>{filename}</span></header>

<div class="card">
  <h2>Candlestick + Swings</h2>
  <div class="tb" id="tb"></div>
  <canvas id="cv"></canvas>
  <div class="legend">
    <span><span class="dot" style="background:var(--hi)"></span>Swing high + confirmation</span>
    <span><span class="dot" style="background:var(--lo)"></span>Swing low + confirmation</span>
    <span><span class="dot" style="background:#2e2e3e"></span>Untagged</span>
  </div>
</div>

<div class="card">
  <h2>Benchmark</h2>
  <table>
    <thead><tr><th>Mode</th><th>Candles</th><th class="hi">Highs</th><th class="lo">Lows</th><th>Ingest</th><th>Detect</th><th>Total</th><th>Speed</th><th>Swing rate</th></tr></thead>
    <tbody>{table_rows}</tbody>
  </table>
</div>

<script>
const CANDLES=[{candle_json}];
const MODES={swing_json};
let active=0;
const cv=document.getElementById('cv');
const ctx=cv.getContext('2d');

function resize(){{
  const w=cv.parentElement.clientWidth-32,h=400,dpr=window.devicePixelRatio||1;
  cv.width=w*dpr;cv.height=h*dpr;cv.style.width=w+'px';cv.style.height=h+'px';
  ctx.scale(dpr,dpr);draw();
}}

function draw(){{
  const W=cv.width/(window.devicePixelRatio||1),H=cv.height/(window.devicePixelRatio||1);
  ctx.clearRect(0,0,W,H);
  if(!CANDLES.length)return;
  const pad={{l:58,r:14,t:14,b:26}};
  const CW=W-pad.l-pad.r,CH=H-pad.t-pad.b;

  let lo=Infinity,hi=-Infinity;
  for(const c of CANDLES){{if(c.l<lo)lo=c.l;if(c.h>hi)hi=c.h;}}
  const swings=MODES[active]?.points||[];
  for(const s of swings){{if(s.p<lo)lo=s.p;if(s.p>hi)hi=s.p;}}
  const rng=(hi-lo)||1;lo-=rng*.05;hi+=rng*.05;
  const TR=hi-lo;
  function py(p){{return pad.t+CH*(1-(p-lo)/TR);}}

  const gap=CW/CANDLES.length;
  const cw=Math.max(1,gap-1);

  // tag candles by nearest timestamp
  const tsArr=CANDLES.map(c=>c.t);
  function nearIdx(ts){{
    let a=0,b=tsArr.length-1;
    while(a<b){{const m=(a+b)>>1;if(tsArr[m]<ts)a=m+1;else b=m;}}
    return a;
  }}
  const tags=new Int8Array(CANDLES.length);
  const WING=2;
  for(const s of swings){{
    const i=nearIdx(s.t);
    if(s.h){{
      tags[i]=1;
      for(let k=1;k<=WING;k++){{if(i-k>=0&&tags[i-k]===0)tags[i-k]=2;if(i+k<tags.length&&tags[i+k]===0)tags[i+k]=2;}}
    }}else{{
      tags[i]=-1;
      for(let k=1;k<=WING;k++){{if(i-k>=0&&tags[i-k]===0)tags[i-k]=-2;if(i+k<tags.length&&tags[i+k]===0)tags[i+k]=-2;}}
    }}
  }}

  function col(tag,bull){{
    if(tag===1) return bull?'#2ecc8f':'#1a9966';
    if(tag===2) return bull?'#1a6644':'#144d33';
    if(tag===-1)return bull?'#e8404f':'#b02030';
    if(tag===-2)return bull?'#6b1a22':'#4d1219';
    return '#2e2e3e';
  }}

  // grid
  ctx.strokeStyle='#1a1a2e';ctx.lineWidth=.5;
  for(let i=0;i<=5;i++){{
    const y=pad.t+(CH/5)*i;
    ctx.beginPath();ctx.moveTo(pad.l,y);ctx.lineTo(pad.l+CW,y);ctx.stroke();
    ctx.fillStyle='#5a576e';ctx.font='10px DM Mono,monospace';ctx.textAlign='right';
    ctx.fillText((hi-(TR/5)*i).toFixed(0),pad.l-4,y+3);
  }}

  // candles
  for(let i=0;i<CANDLES.length;i++){{
    const c=CANDLES[i];
    const x=pad.l+i*gap+gap/2;
    const bull=c.c>=c.o;
    const color=col(tags[i],bull);
    ctx.strokeStyle=color;ctx.lineWidth=1;
    ctx.beginPath();ctx.moveTo(x,py(c.h));ctx.lineTo(x,py(c.l));ctx.stroke();
    const by=py(Math.max(c.o,c.c));
    const bh=Math.max(1,py(Math.min(c.o,c.c))-by);
    ctx.fillStyle=color;ctx.fillRect(x-cw/2,by,cw,bh);
  }}

  // swing markers
  for(const s of swings){{
    const x=pad.l+nearIdx(s.t)*gap+gap/2;
    const y=py(s.p);
    ctx.fillStyle=s.h?'#2ecc8f':'#e8404f';
    ctx.beginPath();
    if(s.h){{ctx.moveTo(x,y-10);ctx.lineTo(x-5,y);ctx.lineTo(x+5,y);}}
    else   {{ctx.moveTo(x,y+10);ctx.lineTo(x-5,y);ctx.lineTo(x+5,y);}}
    ctx.closePath();ctx.fill();
  }}
}}

// toolbar
const tb=document.getElementById('tb');
MODES.forEach((m,i)=>{{
  const b=document.createElement('button');
  b.className='btn'+(i===0?' on':'');
  b.textContent=m.label;
  b.onclick=()=>{{active=i;tb.querySelectorAll('.btn').forEach((x,j)=>x.classList.toggle('on',j===i));draw();}};
  tb.appendChild(b);
}});

window.addEventListener('resize',resize);
resize();
</script>
</body></html>"##,
        filename    = filename,
        candle_json = candle_json,
        swing_json  = swing_json,
        table_rows  = table_rows,
    )
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let data_dir   = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/test/test_data");
    let output_dir = data_dir.join("output");
    fs::create_dir_all(&output_dir).expect("Failed to create output dir");

    let mut csv_files: Vec<PathBuf> = fs::read_dir(&data_dir)
        .expect("Failed to read test_data dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "csv").unwrap_or(false))
        .collect();
    csv_files.sort();

    if csv_files.is_empty() {
        eprintln!("No CSV files found in {:?}", data_dir);
        return;
    }

    println!("\n╔══ AlphaRust Swing Benchmark ═══════════════════════════════╗");
    println!("║  Data:   {:?}", data_dir);
    println!("║  Output: {:?}", output_dir);
    println!("╚════════════════════════════════════════════════════════════╝\n");

    for csv_path in &csv_files {
        let filename = csv_path.file_name().unwrap().to_string_lossy().to_string();
        println!("▶  {}", filename);

        let modes = mode_configs();
        let mut all_results:    Vec<BenchResult>       = Vec::new();
        let mut all_swing_data: Vec<Vec<SwingPoint>>   = Vec::new();
        let mut shared_candles: Vec<Candle>            = Vec::new();

        for mode in &modes {
            print!("   {:25} ... ", mode.label);
            std::io::stdout().flush().ok();

            let (result, candles, swings) =
                run_benchmark(&csv_path, mode.label, (mode.confirmation)());

            println!(
                "{} candles | ↑{} ↓{} | ingest {:.1}ms detect {:.1}ms | {}",
                format_num(result.candles_processed),
                format_num(result.highs_detected),
                format_num(result.lows_detected),
                result.ingest_ms,
                result.detect_ms,
                format_rate(result.candles_per_sec),
            );

            if shared_candles.is_empty() { shared_candles = candles; }
            all_swing_data.push(swings);
            all_results.push(result);
        }

        let html     = generate_html(&filename, &all_results, &all_swing_data, &shared_candles);
        let out_path = output_dir.join(format!("{}.html", filename));
        let mut f    = File::create(&out_path).expect("Failed to create HTML");
        f.write_all(html.as_bytes()).expect("Failed to write HTML");
        println!("   ✓ → {:?}\n", out_path);
    }

    println!("Done. Open the HTML files in src/test/test_data/output/");
}