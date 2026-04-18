
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use dfu_core::{usb, firmware::Firmware, device::DfuHandle, transfer};
use log::error;

#[derive(Parser)]
#[command(name = "dfu-util-rs")]
#[command(about = "A DFU utility written in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all connected DFU devices
    List,

    /// Download firmware to a DFU device
    Download {
        /// Firmware file to flash
        #[arg(short, long)]
        file: String,

        /// USB vendor ID in hex (e.g. 0483)
        #[arg(short, long)]
        vid: String,

        /// USB product ID in hex (e.g. df11)
        #[arg(short, long)]
        pid: String,
    },
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::List => cmd_list(),
        Commands::Download { file, vid, pid } => cmd_download(file, vid, pid),
    };

    if let Err(e) = result {
        error!("error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_list() -> Result<(), dfu_core::error::DfuError> {
    let devices = usb::find_dfu_devices()?;

    if devices.is_empty() {
        println!("no DFU devices found");
        return Ok(());
    }

    println!("found {} DFU device(s):\n", devices.len());

    for dev in &devices {
        println!(
            "  [{:04x}:{:04x}] bus {:03} device {:03} — {} {}",
            dev.vendor_id,
            dev.product_id,
            dev.bus,
            dev.address,
            dev.manufacturer.as_deref().unwrap_or("unknown"),
            dev.product.as_deref().unwrap_or("unknown"),
        );
    }

    Ok(())
}

fn cmd_download(
    file: String,
    vid: String,
    pid: String,
) -> Result<(), dfu_core::error::DfuError> {
    let vid = u16::from_str_radix(vid.trim_start_matches("0x"), 16)
        .map_err(|_| dfu_core::error::DfuError::InvalidFirmware(
            format!("invalid VID: {}", vid)
        ))?;

    let pid = u16::from_str_radix(pid.trim_start_matches("0x"), 16)
        .map_err(|_| dfu_core::error::DfuError::InvalidFirmware(
            format!("invalid PID: {}", pid)
        ))?;

    let firmware = Firmware::load(&file)?;

    println!("loaded {} ({} bytes)", file, firmware.size());

    let handle = DfuHandle::open(vid, pid)?;

    let bar = ProgressBar::new(firmware.size() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner} [{bar:40}] {bytes}/{total_bytes} ({percent}%)")
            .unwrap()
            .progress_chars("=> "),
    );

    transfer::download(&handle, &firmware, |progress| {
        bar.set_position(progress.bytes_sent as u64);
    })?;

    bar.finish_with_message("done");
    println!("firmware flashed successfully");

    Ok(())
}
