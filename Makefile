
.PHONY: rv-test

CROSS_COMPILE = /opt/riscv-gnu-toolchain/install/bin/riscv64-unknown-elf-
#OBJCOPY = $(CROSS_COMPILE)objcopy
OBJCOPY = riscv64-unknown-linux-gnu-objcopy
#RV_TEST_ROOT = /home/froster/code/ics2024/riscv-tests
RV_TEST_ROOT = /home/froster/code/Nemu-rust/riscv-tests/install/share/riscv-tests
TEST = $(RV_TEST_ROOT)/isa/rv64mi-p-access
#TEST = $(RV_TEST_ROOT)/isa/rv64mi-p-csr


binary:
	$(OBJCOPY) -S --set-section-flags .bss=alloc,contents -O binary $(TEST) ./tests/rvtest.bin

rv-test-diff: binary
	cargo run --release --package nemu-rust --bin nemu-rust -- --difftest ./tests/rvtest.bin

rv-test: binary
	cargo run --release --package nemu-rust --bin nemu-rust -- ./tests/rvtest.bin
