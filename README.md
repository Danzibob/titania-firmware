# Titania 3 Firmware

Embedded Rust firmware for the Titania data logger built on the [Embassy](https://embassy.dev)
async framework.

## Hardware target

| | |
|---|---|
| MCU | STM32L072CB (Cortex-M0+) |
| Target triple | `thumbv6m-none-eabi` |
| Flash | 128 KB |
| RAM | 20 KB |
| Sensors | BMI260 (IMU), BMP580 (barometer) on shared I²C |
| External Flash | MX25R3235FM1IL0 (4MB) |
| Other | Capacitive piezo buzzer (no driver) |

The 128 KB flash is the binding constraint — keep an eye on binary size (see
[Building](#building) and [Troubleshooting](#troubleshooting)).

## Prerequisites

A recent stable Rust toolchain (developed against 1.96.0). Install via
[rustup](https://rustup.rs) if you don't have it.

### Toolchain components

```bash
# Bare-metal compilation target for the L072 (Cortex-M0+)
rustup target add thumbv6m-none-eabi

# LLVM tools — needed by cargo-binutils for objcopy/size/nm
rustup component add llvm-tools
```

### Tools

```bash
# ELF -> raw binary conversion (provides `cargo objcopy`, `cargo size`, ...)
cargo install cargo-binutils

# Debug-probe flashing + RTT log viewing (used by `cargo run`)
# Check https://probe.rs for the current install method if this command changes.
cargo install probe-rs-tools --locked
```

For production DFU flashing you also need `dfu-util`:

```bash
# Debian / Ubuntu
sudo apt install dfu-util

# macOS
brew install dfu-util
```

STM32CubeProgrammer is an optional alternative to `dfu-util` for the DFU step.

### Linux: device permissions

To use a debug probe and the DFU bootloader without `sudo`:

- **probe-rs:** install its udev rules — see
  https://probe.rs/docs/getting-started/probe-setup/
- **DFU:** the ST ROM bootloader enumerates as USB ID `0483:df11`. Add a udev rule:

  ```
  # /etc/udev/rules.d/70-st-dfu.rules
  SUBSYSTEM=="usb", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="df11", MODE="0666"
  ```

  Then `sudo udevadm control --reload-rules && sudo udevadm trigger`.

## Building

### Development (with a debug probe)

Building with `debug_assertions` targets a connected probe: defmt logging over RTT and a
panic handler that traps into the debugger.

```bash
cargo build          # debug build
cargo run            # build, flash via probe-rs, and stream defmt logs
```

### Production (no probe, for DFU)

Without `debug_assertions`, the debugger-trap panic handler is swapped for `panic-reset`
(the board reboots on panic instead of hanging).
Release mode also applies size-oriented release optimisations.

```bash
cargo build --release --no-default-features
```

Then convert the ELF to a raw binary for the bootloader:

```bash
cargo objcopy --release --no-default-features \
  -- -O binary titania.bin
```

> [!TIP]
> No `cargo objcopy`? It comes from `cargo-binutils` (see Prerequisites). As a
> fallback you can call `llvm-objcopy -O binary <elf> titania.bin` directly on the
> ELF in `target/thumbv6m-none-eabi/release/`.

## Flashing

### Development — probe-rs (SWD)

`cargo run` handles this via the runner configured in `.cargo/config.toml`
(`probe-rs run --chip STM32L072CBTx`). Just connect the probe and run it.

### Production — USB DFU (no probe)

The STM32L072's built-in ROM bootloader speaks USB DFU, so no probe or pre-flashed
bootloader is needed.

1. Put the board into bootloader mode: pull **BOOT0 high** (to VCC) and reset or
   power-cycle, with the USB cable connected. The chip enumerates as a DfuSe device
   (`0483:df11`).
2. Flash the binary to flash base and start the app:

   ```bash
   dfu-util --list                              # confirm the device appears
   dfu-util -a 0 -s 0x08000000:leave -D titania.bin
   ```

   - `-a 0` selects the internal-flash alternate setting
   - `-s 0x08000000` is the flash base load address (DfuSe needs it explicit)
   - `:leave` exits DFU and boots the freshly flashed app
3. Return BOOT0 to ground so normal resets run the application.

> **Mass flashing:** every board enumerates with the same `0483:df11` ID, so DFU is
> effectively one-board-at-a-time. A jig that cycles power/BOOT0 and runs the
> `dfu-util` line per board is the practical approach. A software DFU trigger (write
> a sentinel to a backup register, reset, jump to system memory on boot) removes the
> need for physical BOOT0 access.

## Logging & panic behaviour

Controlled by whether [`debug_assertions`](https://doc.rust-lang.org/reference/conditional-compilation.html#debug_assertions) are enabled.

`defmt` and `defmt-rtt` are linked in both builds; only the panic handler differs.
The RTT logger is harmless without a probe — it writes into a buffer nobody reads. If
flash gets tight, defmt can be fully feature-gated out of the release build to reclaim
the space.

## Repository layout

```
.cargo/config.toml   # target, runner (probe-rs --chip STM32L072CBTx), link args
.vscode/settings.json # rust-analyzer config (allTargets = false, see below)
memory.x             # flash/RAM regions (generated from the chip feature)
src/main.rs          # firmware entry point
Cargo.toml           # dependencies and feature definitions
```

## Troubleshooting

**`can't find crate for 'test'` flagged by rust-analyzer.** The analyzer checks test
targets, which need libtest — absent on bare-metal. Set
`rust-analyzer.check.allTargets = false` in `.vscode/settings.json`. (Committed in
this repo so it's fixed on clone.)

**`.rodata`/`.data` will not fit in region `FLASH`.** The binary exceeds 128 KB.
Confirm the `embassy-stm32` chip feature is `stm32l072cb` (not a larger part), and that
release builds use size optimisation (`opt-level = "s"`/`"z"`, `lto`, `codegen-units = 1`).

**`no '.defmt' section` from probe-rs.** `-Tdefmt.x` is missing from the linker args —
check `rustflags` in `.cargo/config.toml`, and that no `RUSTFLAGS` env var is shadowing
it.

**`undefined symbol: _defmt_acquire` in release.** A defmt global logger isn't linked.
Keep `defmt-rtt` linked (`use defmt_rtt as _;`) outside the `probe` feature gate, or
fully feature-gate every `defmt::*` call site.
