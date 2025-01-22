
.PHONY: rv-test

CROSS_COMPILE = /opt/riscv-gnu-toolchain/install/bin/riscv64-unknown-elf-
OBJCOPY = $(CROSS_COMPILE)objcopy

RV_TEST_ROOT = /home/froster/code/ics2024/riscv-tests

rv-test:
	$(OBJCOPY) -S --set-section-flags .bss=alloc,contents -O binary $(RV_TEST_ROOT)/isa/rv64mi-p-access ./tests/rvtest.bin
	cargo run --release --package nemu-rust --bin nemu-rust -- --difftest ./tests/rvtest.bin
