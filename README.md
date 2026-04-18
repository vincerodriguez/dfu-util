# dfu-util-rs

A from-scratch implementation of the USB Device Firmware Upgrade (DFU) protocol in Rust. Targets STM32 microcontrollers using the ST DFU bootloader, with a clean library/CLI split so the core protocol can be embedded in other tools.

## Features

- Enumerate all connected DFU-capable USB devices
- Mass erase flash before programming
- Set target start address
- Chunked firmware download with progress bar
- Full DFU state machine handling and automatic error recovery
- Graceful handling of device reset on transfer completion
- Structured logging via `RUST_LOG`

## Project Structure

```
dfu-util-rs/
├── dfu-core/          # Protocol library (no I/O dependencies)
│   └── src/
│       ├── lib.rs
│       ├── error.rs           # DfuError type
│       ├── state_machine.rs   # DfuState and DfuStatus enums
│       ├── protocol.rs        # DFU spec constants and control transfer types
│       ├── usb.rs             # USB device enumeration
│       ├── device.rs          # Device handle, control transfers
│       ├── firmware.rs        # Binary file loading and chunking
│       └── transfer.rs        # Download loop and state machine sequencing
└── dfu-cli/           # Command line interface
    └── src/
        └── main.rs            # clap CLI, progress bar, argument parsing
```

## Requirements

- Rust 1.70 or later (install via [rustup](https://rustup.rs))
- `libusb` development headers

On Debian/Ubuntu/Raspberry Pi OS:

```bash
sudo apt install libusb-1.0-0-dev
```

## Building

```bash
git clone https://github.com/vincerodriguez/dfu-util.git
cd dfu-util
cargo build --release
```

The binary will be at `target/release/dfu-cli`.

## USB Permissions

On Linux, raw USB access requires a udev rule. Without it, the tool must be run as root.

```bash
sudo cp contrib/99-dfu.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Then replug the device. The rule grants access to STM32 devices in DFU mode (`VID 0483`, `PID df11`).

## Usage

### List connected DFU devices

```bash
./target/release/dfu-cli list
```

Example output:

```
found 1 DFU device(s):

  [0483:df11] bus 003 device 009 — STMicroelectronics STM32  BOOTLOADER
```

### Flash firmware

```bash
./target/release/dfu-cli download \
    --file firmware.bin \
    --vid 0483 \
    --pid df11 \
    --address 0x08000000
```

| Flag | Description | Default |
|------|-------------|---------|
| `--file` | Path to `.bin` firmware image | required |
| `--vid` | USB vendor ID (hex) | required |
| `--pid` | USB product ID (hex) | required |
| `--address` | Flash start address (hex) | `0x08000000` |
| `--no-erase` | Skip mass erase before programming | false |

### Verbose logging

Set `RUST_LOG` to see protocol-level detail:

```bash
RUST_LOG=debug ./target/release/dfu-cli download --file firmware.bin --vid 0483 --pid df11
RUST_LOG=info  ./target/release/dfu-cli download --file firmware.bin --vid 0483 --pid df11
```

## Firmware Format

Pass a raw `.bin` file — the bytes that get written to flash verbatim starting at `--address`. Do not pass `.elf` or `.hex` files.

To convert an ELF to a binary using the ARM toolchain:

```bash
arm-none-eabi-objcopy -O binary firmware.elf firmware.bin
```

## Putting the STM32 into DFU Mode

The STM32F4 enters DFU mode when BOOT0 is pulled high on reset. On most development boards this means:

1. Hold the BOOT0 button (or bridge the BOOT0 jumper)
2. Press and release RESET
3. Release BOOT0

The device will enumerate as `[0483:df11]`. Confirm with `dfu-cli list`.

## Protocol Notes

This tool implements the USB DFU 1.1 specification with ST's DFU extension protocol for address setting and mass erase. The ST bootloader uses block 0 as a command channel rather than a data channel — set-address and erase commands are sent as zero-wValue block 0 downloads before the actual firmware transfer begins at block 2.

The zero-length termination block causes the STM32 to immediately reset and boot the new firmware, which the host sees as an I/O error. This is expected behavior and is handled gracefully.

## Crates Used

| Crate | Purpose |
|-------|---------|
| [`rusb`](https://crates.io/crates/rusb) | libusb bindings for USB device access |
| [`thiserror`](https://crates.io/crates/thiserror) | Ergonomic error type derivation |
| [`clap`](https://crates.io/crates/clap) | Command line argument parsing |
| [`indicatif`](https://crates.io/crates/indicatif) | Progress bar |
| [`log`](https://crates.io/crates/log) | Structured logging facade |
| [`env_logger`](https://crates.io/crates/env_logger) | Log output controlled by `RUST_LOG` |

## License

MIT