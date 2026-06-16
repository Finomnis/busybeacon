# BusyBeacon Firmware

## Datasheets:

- [STM32U073CCU6 Datasheet](https://www.st.com/resource/en/datasheet/stm32u073cc.pdf)
- [STM32U0 Reference Manual](https://www.st.com/resource/en/reference_manual/rm0503-stm32u0-series-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)

## Getting Started

Before getting started, you need Rust/Cargo installed.

See https://rustup.rs/.

### Further dependencies

- `cargo binstall` - Optional, if not desired replace `binstall` in the other dependencies with `install`

  ```shell
  cargo install cargo-binstall --locked
  ```

- `flip-link`

  ```shell
  cargo binstall flip-link
  ```

- `cargo objcopy`

  ```
  rustup component add llvm-tools
  cargo binstall cargo-binutils
  ```

- `probe-rs`

  ```
  cargo binstall probe-rs-tools
  ```

## Application

### Build

```shell
cargo build --release
```

### Build and flash

```shell
cargo program --release
```

### Generate hex

```shell
cargo objcopy --release -- -O ihex app.hex

# To flash:
probe-rs download --binary-format hex --chip stm32u073cc app.hex
```

### Generate dfu file

```shell
cargo objcopy --release -- -O binary app.dfu
# Add dfu suffix containing VID:PID to the image:
dfu-suffix --vid 1209 --pid d9d0 --add app.dfu

# To update:
sudo dfu-util --download app.dfu
```

*Note*: It is normal to get the following error at the end of the update.
This is currently a technical limitation, but it's purely cosmetic.

```text
...
Download done.
unable to read DFU status after completion (LIBUSB_ERROR_PIPE)
```


## Bootloader


### Build

```shell
cargo build -p busybeacon-bootloader --release
```

### Build and flash

```shell
cargo program -p busybeacon-bootloader --release
```

### Generate hex

```shell
cargo objcopy -p busybeacon-bootloader --release -- -O ihex bootloader.hex

# To flash:
probe-rs download --binary-format hex --chip stm32u073cc bootloader.hex
```

## Tests

### Execute tests on host

```shell
cargo test-host
```
