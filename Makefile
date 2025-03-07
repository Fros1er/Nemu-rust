
.PHONY: build-rv-test rv-test rv-test-one build_linux linux 

LOG := info
OBJCOPY = riscv64-unknown-linux-gnu-objcopy
RV_TEST_ROOT = ./riscv-tests/install/share/riscv-tests/isa

# 查找 TEST_DIR 下所有以 rv64- 开头的文件
RV_TEST_BINS := $(patsubst $(RV_TEST_ROOT)/%,$(RV_TEST_ROOT)/binary/%.bin,$(filter-out %.dump,$(wildcard $(RV_TEST_ROOT)/rv64*)))

# 仅匹配 rv64- 开头的文件
$(RV_TEST_ROOT)/binary/rv64%.bin: $(filter-out $(RV_TEST_ROOT)/rv64%.dump,$(RV_TEST_ROOT)/rv64%)
	$(OBJCOPY) -S --set-section-flags .bss=alloc,contents -O binary $< $@

# 目标规则
build-rv-test: $(RV_TEST_BINS)

EXCLUDED_BINS := rv64mi-p-illegal.bin rv64mi-p-breakpoint.bin rv64mi-p-ma_fetch.bin \
	rv64mi-p-zicntr.bin rv64si-p-ma_fetch.bin rv64si-p-wfi.bin

RV_TEST_BINS_FINAL := $(filter-out $(addprefix $(RV_TEST_ROOT)/binary/, $(EXCLUDED_BINS)), $(RV_TEST_BINS))

rv-test: $(RV_TEST_BINS)
	@for bin in $(RV_TEST_BINS_FINAL); do \
  		FILENAME=$$(basename $$bin); \
		cargo run --release --package nemu-rust --bin nemu-rust -- \
			--batch --log-level=warn --term-timeout=0 --firmware $$bin || exit 1; \
		echo "\033[32mrv-test $$FILENAME successful!\033[0m"; \
	done

DIFFTEST :=
DIFFTEST_$(DIFFTEST) := --difftest

rv-test-one: 
	@if [ -z "$(BIN)" ]; then \
		echo "Please specify the test file using 'make rv-test-one BIN=rv64xxx.bin'"; \
		exit 1; \
	fi
	@echo "Running test for $(BIN)..."
	cargo run --release --package nemu-rust --bin nemu-rust -- $(DIFFTEST_y) \
		--log-level=$(LOG) --term-timeout=0 --firmware $(RV_TEST_ROOT)/binary/$(BIN)

opensbi:
	cargo run --release --package nemu-rust --bin nemu-rust -- --ignore-isa-breakpoint --firmware opensbi-1.6/build/platform/generic/firmware/fw_jump.bin ./tests/rvtest.bin

build_linux: nemu-rust.dtb
	$(MAKE) -C linux/linux ARCH=riscv CROSS_COMPILE=riscv64-unknown-linux-gnu- -j14
	$(MAKE) -C opensbi-1.6 clean
	$(MAKE) -C opensbi-1.6 CROSS_COMPILE=riscv64-unknown-linux-gnu- PLATFORM=generic PLATFORM_RISCV_ISA=rv64ima_zicsr_zifencei FW_TEXT_START=0x80000000 FW_PAYLOAD_PATH=$(CURDIR)/linux/linux/arch/riscv/boot/Image FW_FDT_PATH=$(CURDIR)/nemu-rust.dtb FW_PAYLOAD_FDT_ADDR=0x9ff00000 -j14	

linux:
	cargo run --release --package nemu-rust --bin nemu-rust -- --log-level=$(LOG) --term-timeout=0 --ignore-isa-breakpoint --firmware opensbi-1.6/build/platform/generic/firmware/fw_payload.bin 

sustechos:
	cargo run --release --package nemu-rust --bin nemu-rust -- --log-level=$(LOG) --ignore-isa-breakpoint --firmware opensbi-1.6/build/platform/generic/firmware/fw_jump.bin ./SUSTechOS/build/kernel.bin

sustechos-batch:
	cargo run --release --package nemu-rust --bin nemu-rust -- --log-level=$(LOG) --batch --ignore-isa-breakpoint --firmware opensbi-1.6/build/platform/generic/firmware/fw_jump.bin --image ./SUSTechOS/build/kernel.bin


build_opensbi: nemu-rust.dtb
	cd opensbi-1.6 && make CROSS_COMPILE=riscv64-unknown-linux-gnu- PLATFORM=generic PLATFORM_RISCV_ISA=rv64ima_zicsr_zifencei FW_TEXT_START=0x80000000 FW_JUMP_ADDR=0x80200000 FW_FDT_PATH=../nemu-rust.dtb FW_JUMP_FDT_ADDR=0x89000000 -j6
	cd opensbi-1.6 && riscv64-unknown-linux-gnu-objdump -d build/platform/generic/firmware/fw_jump.elf > disasm	

nemu-rust.dtb: nemu-rust.dts
	dtc -I dts -O dtb -o nemu-rust.dtb nemu-rust.dts
