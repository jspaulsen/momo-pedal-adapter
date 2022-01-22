# Momo Pedal Adapter

Adapter software for the Pico for creating a Momo Pedal Adapter written in Rust.

## Momo Pinout

| DB9 | Pico               |
|-----|--------------------|
| 2   | GP28 (Accelerator) |
| 3   | GP27 (Brake)       |
| 4   | ADC_VREF           |
| 6   | GND                |

![Pinout](https://external-preview.redd.it/S5BflE95IkjjjeUkvn9LgeV4Imw2jHGZRd8wbTi6-g4.jpg?auto=webp&s=a13e78acedd8437eace6cc4d84bc1272ba38f1a7)

## Scaling and Values

In experimenting, a lot of "slop" was found in both pedals; minimum values hovered around 700.  The software compensates for this, but values
likely vary between hardware so YMMV.

## Building
###  Requirements
- The standard Rust tooling (cargo, rustup) which you can install from https://rustup.rs/

- Toolchain support for the cortex-m0+ processors in the rp2040 (thumbv6m-none-eabi)

- flip-link - this allows you to detect stack-overflows on the first core, which is the only supported target for now.

- probe-run. Upstream support for RP2040 was added with version 0.3.1.

- A CMSIS-DAP probe. (JLink probes sort of work but are very unstable. Other probes won't work at all)

  You can use a second Pico as a CMSIS-DAP debug probe by installing the following firmware on it:
  https://github.com/majbthrd/DapperMime/releases/download/20210225/raspberry_pi_pico-DapperMime.uf2

  More details on supported debug probes can be found in [debug_probes.md](debug_probes.md)

### Installation of development dependencies
```
rustup target install thumbv6m-none-eabi
cargo install flip-link
# This is our suggested default 'runner'
cargo install probe-run
# If you want to use elf2uf2-rs instead of probe-run, instead do...
cargo install elf2uf2-rs
```

### Running

For a debug build
```
DEFMT_LOG=trace cargo run
```
For a release build
```
DEFMT_LOG=trace cargo run --release
```

To load firmware directly into RP2040, for example you don't have a probe, comment and uncomment following lines in `.cargo/config.toml`:

```
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
# runner = "probe-run --chip RP2040"
runner = "elf2uf2-rs -d"
```
