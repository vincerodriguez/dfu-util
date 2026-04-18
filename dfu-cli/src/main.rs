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
        /// Firmware file to flash (.bin only)
        #[arg(short, long)]
        file: String,

        /// USB vendor ID in hex (e.g. 0483)
        #[arg(short, long)]
        vid: String,

        /// USB product ID in hex (e.g. df11)
        #[arg(short, long)]
        pid: String,

        /// Start address in hex (default: 0x08000000)
        #[arg(short, long, default_value = "0x08000000")]
        address: String,

        /// Skip mass erase before programming
        #[arg(long, default_value_t = false)]
        no_erase: bool,
    },
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::List => cmd_list(),
        Commands::Download { file, vid, pid, address, no_erase } => {
            cmd_download(file, vid, pid, address, no_erase)
        }
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

fn parse_hex_u32(s: &str) -> Result<u32, dfu_core::error::DfuError> {
    u32::from_str_radix(s.trim_start_matches("0x"), 16)
        .map_err(|_| dfu_core::error::DfuError::InvalidFirmware(
            format!("invalid hex value: {}", s)
        ))
}

fn parse_hex_u16(s: &str) -> Result<u16, dfu_core::error::DfuError> {
    u16::from_str_radix(s.trim_start_matches("0x"), 16)
        .map_err(|_| dfu_core::error::DfuError::InvalidFirmware(
            format!("invalid hex value: {}", s)
        ))
}

fn cmd_download(
    file: String,
    vid: String,
    pid: String,
    address: String,
    no_erase: bool,
) -> Result<(), dfu_core::error::DfuError> {
    let vid = parse_hex_u16(&vid)?;
    let pid = parse_hex_u16(&pid)?;
    let start_address = parse_hex_u32(&address)?;

    let firmware = Firmware::load(&file)?;

    println!("loaded {} ({} bytes)", file, firmware.size());
    println!("target address: {:#010x}", start_address);
    println!("erase: {}", if no_erase { "skipped" } else { "mass erase" });

    let handle = DfuHandle::open(vid, pid)?;

    let bar = ProgressBar::new(firmware.size() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner} [{bar:40}] {bytes}/{total_bytes} ({percent}%)")
            .unwrap()
            .progress_chars("=> "),
    );

    transfer::download(&handle, &firmware, start_address, no_erase, |progress| {
        bar.set_position(progress.bytes_sent as u64);
    })?;

    bar.finish_with_message("done");
    println!("firmware flashed successfully");

    Ok(())
}
