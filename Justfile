#!/usr/bin/env -S just --justfile

# build, flash and run a debug build
debug *args:
    cargo run {{ args }}

# build, flash and run a release build
release-probe *args:
    cargo run --release {{ args }}

DFU_ARTEFACT := "titania.bin"

# build a DFU-flashable release at DFU_ARTEFACT
release-dfu *args:
    cargo objcopy --release {{ args }} -- -O binary "{{ DFU_ARTEFACT }}"
    du -h "{{ DFU_ARTEFACT }}"

# build and flash a release image over DFU
[confirm("Have you pulled BOOT0 high and connected the board [y/N]?")]
flash-dfu flash_addr="0x08000000" *build-args: (release-dfu build-args)
    dfu-util -a0 -s "{{ flash_addr }}:leave" -D "{{ DFU_ARTEFACT }}"
