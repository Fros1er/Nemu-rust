# Nemu-rust

RISC-V emulator in rust.

following https://github.com/NJU-ProjectN/ics-pa#/

abstract-machine is sourced from https://github.com/NJU-ProjectN/abstract-machine

~~microbench: 3048(native 63692)~~ Outdated.
PAL running at ~10 fps.

compile opensbi with
`make CROSS_COMPILE=riscv64-unknown-linux-gnu- PLATFORM=generic PLATFORM_RISCV_ISA=rv64imafd_zicsr_zifencei`.