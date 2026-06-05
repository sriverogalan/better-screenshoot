use std::fs;
use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD, Engine};
use clap::{Parser, Subcommand};
use capture_core::{create_provider, Region};
use capture_core::CaptureProvider;

#[derive(Parser)]
#[command(name = "better-screenshoot")]
#[command(about = "Better Screenshoot CLI — capture and integrate from the terminal")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Trigger capture via URL scheme (opens the desktop app)
    Open {
        /// Action: capture-area, capture-screen, capture-window, open-history, open-settings
        action: String,
    },
    /// Headless capture (no GUI) — saves PNG to file
    Capture {
        #[command(subcommand)]
        mode: CaptureMode,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Output result as JSON
        #[arg(long)]
        json: bool,
    },
    /// List available displays
    Displays,
    /// List available windows
    Windows,
}

#[derive(Subcommand)]
enum CaptureMode {
    /// Capture full screen
    Screen {
        #[arg(long)]
        display: Option<u32>,
    },
    /// Capture a window by ID
    Window {
        id: u64,
    },
    /// Capture a region
    Region {
        #[arg(long)]
        display: u32,
        #[arg(long)]
        x: i32,
        #[arg(long)]
        y: i32,
        #[arg(long)]
        width: u32,
        #[arg(long)]
        height: u32,
    },
}

fn default_output() -> PathBuf {
    let dir = dirs::picture_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("BetterScreenshoot");
    let _ = fs::create_dir_all(&dir);
    let filename = format!("cli-capture-{}.png", chrono_lite_timestamp());
    dir.join(filename)
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs.to_string()
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Open { action } => {
            let url = format!("betterscreenshoot://{}", action.trim_matches('/'));
            open::that(&url).expect("failed to open URL scheme");
            println!("Triggered: {url}");
        }
        Commands::Displays => {
            let provider = create_provider();
            let displays = provider.list_displays().expect("failed to list displays");
            println!("{}", serde_json::to_string_pretty(&displays).unwrap());
        }
        Commands::Windows => {
            let provider = create_provider();
            let windows = provider.list_windows().expect("failed to list windows");
            println!("{}", serde_json::to_string_pretty(&windows).unwrap());
        }
        Commands::Capture { mode, output, json } => {
            let provider = create_provider();
            let image = match mode {
                CaptureMode::Screen { display } => match display {
                    Some(id) => provider.capture_display(id),
                    None => provider.capture_primary_display(),
                }
                .expect("screen capture failed"),
                CaptureMode::Window { id } => {
                    provider.capture_window(id).expect("window capture failed")
                }
                CaptureMode::Region {
                    display,
                    x,
                    y,
                    width,
                    height,
                } => provider
                    .capture_region(display, Region { x, y, width, height })
                    .expect("region capture failed"),
            };

            let out = output.unwrap_or_else(default_output);
            fs::write(&out, &image.png_bytes).expect("failed to write output");

            if json {
                let payload = serde_json::json!({
                    "path": out.to_string_lossy(),
                    "width": image.width,
                    "height": image.height,
                    "base64": STANDARD.encode(&image.png_bytes),
                });
                println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            } else {
                println!("Saved: {}", out.display());
            }
        }
    }
}
