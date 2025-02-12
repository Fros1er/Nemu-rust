# Nemu-rust

RISC-V emulator in rust.

following https://github.com/NJU-ProjectN/ics-pa#/

abstract-machine is sourced from https://github.com/NJU-ProjectN/abstract-machine

~~microbench: 3048(native 63692)~~ Outdated.
PAL running at ~10 fps.

compile opensbi with
```make CROSS_COMPILE=riscv64-unknown-linux-gnu- \
    PLATFORM=generic PLATFORM_RISCV_ISA=rv64imafd_zicsr_zifencei \
    FW_TEXT_START=0x10000 FW_JUMP_ADDR=0x80000000 \
    FW_FDT_PATH=../nemu-rust.dtb FW_JUMP_FDT_ADDR=0x89000000
```

.