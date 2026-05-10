"""
viz.py — AlphaRust Swing Visualizer
Place at: project root (next to Cargo.toml)

Install deps:
    pip install dash plotly pandas

Run:
    python viz.py

Then open: http://localhost:8050
"""

import json
import os
import glob
from pathlib import Path

import dash
from dash import dcc, html, Input, Output, State
import plotly.graph_objects as go
from plotly.subplots import make_subplots

# ── Config ────────────────────────────────────────────────────────────────────
import sys
print(f"Python: {sys.version}", flush=True)
print("Starting server...", flush=True)
JSON_DIR = Path(__file__).parent / "src" / "test" / "test_data" / "output" / "json"

# Colors
COLOR_HIGH        = "#2ecc8f"
COLOR_HIGH_CONF   = "#1a6644"
COLOR_LOW         = "#e8404f"
COLOR_LOW_CONF    = "#6b1a22"
COLOR_NEUTRAL_BULL = "#2e3a2e"
COLOR_NEUTRAL_BEAR = "#3a2e2e"
COLOR_BG          = "#080810"
COLOR_SURFACE     = "#0f0f1a"
COLOR_BORDER      = "#1a1a2e"
COLOR_TEXT        = "#dddaf0"
COLOR_MUTED       = "#5a576e"
COLOR_ACCENT      = "#7c6af7"

CONF_WING = 2  # candles on each side of a swing to color as confirmation

# ── Load JSON files ───────────────────────────────────────────────────────────

def load_json_files():
    files = sorted(glob.glob(str(JSON_DIR / "*.json")))
    result = {}
    for f in files:
        with open(f) as fh:
            data = json.load(fh)
        result[data["file"]] = data
    return result

# ── Build chart ───────────────────────────────────────────────────────────────

def build_chart(data: dict, mode_label: str) -> go.Figure:
    candles = data["candles"]
    mode    = next((m for m in data["modes"] if m["label"] == mode_label), data["modes"][0])

    timestamps = [c["t"] for c in candles]
    opens      = [c["o"] for c in candles]
    highs      = [c["h"] for c in candles]
    lows       = [c["l"] for c in candles]
    closes     = [c["c"] for c in candles]

    # Build swing lookup: timestamp -> type
    swing_map = {}
    for s in mode["swings"]:
        swing_map[s["t"]] = s["type"]  # "high" or "low"

    # Tag each candle index
    ts_to_idx = {t: i for i, t in enumerate(timestamps)}
    tags = [0] * len(candles)  # 0=neutral, 1=high, 2=high_conf, -1=low, -2=low_conf

    for s in mode["swings"]:
        idx = ts_to_idx.get(s["t"])
        if idx is None:
            continue
        if s["type"] == "high":
            tags[idx] = 1
            for k in range(1, CONF_WING + 1):
                if idx - k >= 0 and tags[idx - k] == 0:
                    tags[idx - k] = 2
                if idx + k < len(tags) and tags[idx + k] == 0:
                    tags[idx + k] = 2
        else:
            tags[idx] = -1
            for k in range(1, CONF_WING + 1):
                if idx - k >= 0 and tags[idx - k] == 0:
                    tags[idx - k] = -2
                if idx + k < len(tags) and tags[idx + k] == 0:
                    tags[idx + k] = -2

    def candle_color(tag, is_bull):
        if tag == 1:  return COLOR_HIGH
        if tag == 2:  return COLOR_HIGH_CONF
        if tag == -1: return COLOR_LOW
        if tag == -2: return COLOR_LOW_CONF
        return COLOR_NEUTRAL_BULL if is_bull else COLOR_NEUTRAL_BEAR

    colors = [candle_color(tags[i], closes[i] >= opens[i]) for i in range(len(candles))]

    fig = make_subplots(rows=1, cols=1)

    # Candlestick trace (colored per-candle via marker colors)
    fig.add_trace(go.Candlestick(
        x=timestamps,
        open=opens,
        high=highs,
        low=lows,
        close=closes,
        increasing=dict(line=dict(color=COLOR_NEUTRAL_BULL, width=1), fillcolor=COLOR_NEUTRAL_BULL),
        decreasing=dict(line=dict(color=COLOR_NEUTRAL_BEAR, width=1), fillcolor=COLOR_NEUTRAL_BEAR),
        name="Candles",
        showlegend=False,
        hoverinfo="skip",
    ))

    # Overlay colored OHLC bars per tag group to get per-candle colors
    # (Plotly doesn't support per-candle colors natively on Candlestick,
    #  so we use transparent scatter rectangles via separate traces per color group)
    tag_groups = {1: [], 2: [], -1: [], -2: []}
    for i, tag in enumerate(tags):
        if tag != 0:
            tag_groups[tag].append(i)

    tag_meta = {
        1:  (COLOR_HIGH,      "Swing High"),
        2:  (COLOR_HIGH_CONF, "High Confirm"),
        -1: (COLOR_LOW,       "Swing Low"),
        -2: (COLOR_LOW_CONF,  "Low Confirm"),
    }

    for tag, indices in tag_groups.items():
        if not indices:
            continue
        color, name = tag_meta[tag]
        fig.add_trace(go.Candlestick(
            x=[timestamps[i] for i in indices],
            open=[opens[i]  for i in indices],
            high=[highs[i]  for i in indices],
            low=[lows[i]    for i in indices],
            close=[closes[i] for i in indices],
            increasing=dict(line=dict(color=color, width=1), fillcolor=color),
            decreasing=dict(line=dict(color=color, width=1), fillcolor=color),
            name=name,
            showlegend=True,
        ))

    # Swing markers
    high_swings = [s for s in mode["swings"] if s["type"] == "high"]
    low_swings  = [s for s in mode["swings"] if s["type"] == "low"]

    if high_swings:
        fig.add_trace(go.Scatter(
            x=[s["t"] for s in high_swings],
            y=[s["p"] for s in high_swings],
            mode="markers",
            marker=dict(symbol="triangle-up", size=10, color=COLOR_HIGH),
            name="Swing High",
            showlegend=False,
        ))

    if low_swings:
        fig.add_trace(go.Scatter(
            x=[s["t"] for s in low_swings],
            y=[s["p"] for s in low_swings],
            mode="markers",
            marker=dict(symbol="triangle-down", size=10, color=COLOR_LOW),
            name="Swing Low",
            showlegend=False,
        ))

    # Stats annotation
    n_high = len(high_swings)
    n_low  = len(low_swings)
    total  = data["candle_count"]
    rate   = (n_high + n_low) / total * 100 if total else 0

    fig.update_layout(
        paper_bgcolor=COLOR_BG,
        plot_bgcolor=COLOR_SURFACE,
        font=dict(family="DM Mono, monospace", color=COLOR_TEXT, size=11),
        xaxis=dict(
            rangeslider=dict(visible=False),
            gridcolor=COLOR_BORDER,
            linecolor=COLOR_BORDER,
            type="linear",
        ),
        yaxis=dict(
            gridcolor=COLOR_BORDER,
            linecolor=COLOR_BORDER,
            side="right",
        ),
        legend=dict(
            bgcolor=COLOR_SURFACE,
            bordercolor=COLOR_BORDER,
            borderwidth=1,
            font=dict(size=10),
        ),
        margin=dict(l=10, r=60, t=40, b=10),
        title=dict(
            text=f"↑{n_high} highs  ↓{n_low} lows  |  {rate:.1f}% swing rate  |  {total:,} candles",
            font=dict(size=12, color=COLOR_MUTED),
            x=0,
        ),
        dragmode="zoom",
        hovermode="x unified",
    )

    return fig


def build_stats_table(data: dict) -> html.Table:
    rows = []
    for m in data["modes"]:
        swings   = m["swings"]
        n_high   = sum(1 for s in swings if s["type"] == "high")
        n_low    = sum(1 for s in swings if s["type"] == "low")
        total    = n_high + n_low
        rate     = total / data["candle_count"] * 100 if data["candle_count"] else 0
        cps      = m["candles_per_sec"]
        cps_str  = f"{cps/1_000_000:.2f}M/s" if cps >= 1_000_000 else f"{cps/1_000:.1f}K/s"

        rows.append(html.Tr([
            html.Td(m["label"],             style={"color": COLOR_ACCENT}),
            html.Td(f"{data['candle_count']:,}"),
            html.Td(f"{n_high}",            style={"color": COLOR_HIGH}),
            html.Td(f"{n_low}",             style={"color": COLOR_LOW}),
            html.Td(f"{m['ingest_ms']:.1f}ms"),
            html.Td(f"{m['detect_ms']:.1f}ms"),
            html.Td(f"{m['ingest_ms']+m['detect_ms']:.1f}ms"),
            html.Td(cps_str),
            html.Td(f"{rate:.2f}%"),
        ]))

    header = html.Tr([
        html.Th(h) for h in
        ["Mode", "Candles", "↑ Highs", "↓ Lows", "Ingest", "Detect", "Total", "Speed", "Swing rate"]
    ])

    return html.Table(
        [html.Thead(header), html.Tbody(rows)],
        style={"width": "100%", "borderCollapse": "collapse", "fontSize": "11px"}
    )


# ── App layout ────────────────────────────────────────────────────────────────

app = dash.Dash(__name__, title="AlphaRust Visualizer")
app.layout = html.Div(style={
    "background": COLOR_BG,
    "minHeight": "100vh",
    "padding": "24px",
    "fontFamily": "DM Mono, monospace",
    "color": COLOR_TEXT,
}, children=[

    # Header
    html.Div(style={"borderBottom": f"1px solid {COLOR_BORDER}", "paddingBottom": "14px", "marginBottom": "20px", "display": "flex", "alignItems": "baseline", "gap": "14px"}, children=[
        html.H1("AlphaRust", style={"fontFamily": "sans-serif", "fontSize": "20px", "color": COLOR_ACCENT, "margin": 0}),
        html.Span("Swing Visualizer", style={"color": COLOR_MUTED, "fontSize": "12px"}),
    ]),

    # Controls row
    html.Div(style={"display": "flex", "gap": "16px", "marginBottom": "16px", "flexWrap": "wrap", "alignItems": "center"}, children=[
        html.Div([
            html.Label("Dataset", style={"color": COLOR_MUTED, "fontSize": "11px", "display": "block", "marginBottom": "4px"}),
            dcc.Dropdown(
                id="file-select",
                options=[],
                value=None,
                style={"width": "240px", "fontSize": "12px"},
            ),
        ]),
        html.Div([
            html.Label("Confirmation mode", style={"color": COLOR_MUTED, "fontSize": "11px", "display": "block", "marginBottom": "4px"}),
            dcc.Dropdown(
                id="mode-select",
                options=[],
                value=None,
                style={"width": "260px", "fontSize": "12px"},
            ),
        ]),
        html.Button("↺ Reload files", id="reload-btn", n_clicks=0, style={
            "background": COLOR_SURFACE,
            "border": f"1px solid {COLOR_BORDER}",
            "color": COLOR_MUTED,
            "padding": "6px 14px",
            "borderRadius": "4px",
            "cursor": "pointer",
            "fontSize": "11px",
            "fontFamily": "DM Mono, monospace",
            "marginTop": "16px",
        }),
    ]),

    # Chart
    html.Div(style={"background": COLOR_SURFACE, "border": f"1px solid {COLOR_BORDER}", "borderRadius": "8px", "padding": "12px", "marginBottom": "16px"}, children=[
        dcc.Graph(
            id="chart",
            config={"scrollZoom": True, "displayModeBar": True, "modeBarButtonsToRemove": ["autoScale2d"]},
            style={"height": "500px"},
        ),
    ]),

    # Stats table
    html.Div(style={"background": COLOR_SURFACE, "border": f"1px solid {COLOR_BORDER}", "borderRadius": "8px", "padding": "16px"}, children=[
        html.H2("Benchmark — All Modes", style={"fontFamily": "sans-serif", "fontSize": "13px", "marginBottom": "12px"}),
        html.Div(id="stats-table"),
    ]),

    # Hidden store
    dcc.Store(id="data-store"),
])


# ── Callbacks ─────────────────────────────────────────────────────────────────

@app.callback(
    Output("file-select", "options"),
    Output("file-select", "value"),
    Output("data-store",  "data"),
    Input("reload-btn",   "n_clicks"),
)
def load_files(_):
    all_data = load_json_files()
    if not all_data:
        return [], None, {}
    options = [{"label": k, "value": k} for k in all_data]
    first   = list(all_data.keys())[0]
    return options, first, all_data


@app.callback(
    Output("mode-select", "options"),
    Output("mode-select", "value"),
    Input("file-select",  "value"),
    State("data-store",   "data"),
)
def update_modes(file_key, all_data):
    if not file_key or not all_data or file_key not in all_data:
        return [], None
    modes   = all_data[file_key]["modes"]
    options = [{"label": m["label"], "value": m["label"]} for m in modes]
    return options, modes[0]["label"]


@app.callback(
    Output("chart",       "figure"),
    Output("stats-table", "children"),
    Input("file-select",  "value"),
    Input("mode-select",  "value"),
    State("data-store",   "data"),
)
def update_chart(file_key, mode_label, all_data):
    empty_fig = go.Figure(layout=dict(paper_bgcolor=COLOR_BG, plot_bgcolor=COLOR_SURFACE))
    if not file_key or not mode_label or not all_data or file_key not in all_data:
        return empty_fig, ""
    data  = all_data[file_key]
    fig   = build_chart(data, mode_label)
    table = build_stats_table(data)
    return fig, table


# ── Entry point ───────────────────────────────────────────────────────────────

if __name__ == "__main__":
    print("\n╔══ AlphaRust Visualizer ═════════════════════════════════════╗")
    print(f"║  JSON dir: {JSON_DIR}")
    print( "║  Open:     http://localhost:8050")
    print( "╚════════════════════════════════════════════════════════════╝\n")

    if not JSON_DIR.exists():
        print(f"⚠  JSON dir not found. Run `cargo run --bin export --release` first.\n")

    app.run(debug=True, port=8050)