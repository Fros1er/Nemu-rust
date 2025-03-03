
.PHONY: rv-test build_linux linux

LOG := info
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

opensbi:
	cargo run --release --package nemu-rust --bin nemu-rust -- --ignore-isa-breakpoint --firmware opensbi-1.6/build/platform/generic/firmware/fw_jump.bin ./tests/rvtest.bin

build_linux:
	$(MAKE) -C linux/linux ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- -j14
	$(MAKE) -C opensbi-1.6 clean
	$(MAKE) -C opensbi-1.6 CROSS_COMPILE=riscv64-unknown-linux-gnu- PLATFORM=generic PLATFORM_RISCV_ISA=rv64ima_zicsr_zifencei FW_TEXT_START=0x80000000 FW_PAYLOAD_PATH=$(CURDIR)/linux/linux/arch/riscv/boot/Image FW_FDT_PATH=$(CURDIR)/nemu-rust.dtb FW_PAYLOAD_FDT_ADDR=0x9ff00000 -j14	

linux:
	cargo run --release --package nemu-rust --bin nemu-rust -- --log-level=$(LOG) --ignore-isa-breakpoint --firmware opensbi-1.6/build/platform/generic/firmware/fw_payload.bin

sustechos:
	cargo run --release --package nemu-rust --bin nemu-rust -- --log-level=$(LOG) --ignore-isa-breakpoint --firmware opensbi-1.6/build/platform/generic/firmware/fw_jump.bin ./SUSTechOS/build/kernel.bin

sustechos-batch:
	cargo run --release --package nemu-rust --bin nemu-rust -- --log-level=$(LOG) --batch --ignore-isa-breakpoint --firmware opensbi-1.6/build/platform/generic/firmware/fw_jump.bin --image ./SUSTechOS/build/kernel.bin


build_opensbi:
	dtc -I dts -O dtb -o nemu-rust.dtb nemu-rust.dts
	cd opensbi-1.6 && make CROSS_COMPILE=riscv64-unknown-linux-gnu- PLATFORM=generic PLATFORM_RISCV_ISA=rv64ima_zicsr_zifencei FW_TEXT_START=0x80000000 FW_JUMP_ADDR=0x80200000 FW_FDT_PATH=../nemu-rust.dtb FW_JUMP_FDT_ADDR=0x89000000 -j6
	cd opensbi-1.6 && riscv64-unknown-linux-gnu-objdump -d build/platform/generic/firmware/fw_jump.elf > disasm	

rv-test: binary
	cargo run --release --package nemu-rust --bin nemu-rust -- ./tests/rvtest.bin
