# Nemu-rust

RISC-V emulator in rust, capable to boot linux kernel(v5.4) with initramfs.

inspired by https://github.com/NJU-ProjectN/ics-pa#/

microbench: 475 Marks (vs i9-9900K@3.60GHz's 100000 Marks)  
PAL running at ~10 fps.  
Coremark 1.0 inside kernel: ~25.4 (vs local AMD Ryzen 7 3700X(16)@3.600GHz's 29662.5)

## Usage

1. clone linux v5.4 to ./linux/linux/
2. build linux with ./linux.config
3. build rootfs with busybox to ./linux/rootfs/
4. clone opensbi 1.6 to ./opensbi-1.6/
5. make build_linux && make linux