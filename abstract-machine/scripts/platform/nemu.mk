AM_SRCS := platform/nemu/trm.c \
           platform/nemu/ioe/ioe.c \
           platform/nemu/ioe/timer.c \
           platform/nemu/ioe/input.c \
           platform/nemu/ioe/gpu.c \
           platform/nemu/ioe/audio.c \
           platform/nemu/ioe/disk.c \
           platform/nemu/mpe.c

CFLAGS    += -fdata-sections -ffunction-sections
LDFLAGS   += -T $(AM_HOME)/scripts/linker.ld \
             --defsym=_pmem_start=0x80000000 --defsym=_entry_offset=0x0
LDFLAGS   += --gc-sections -e _start
NEMUFLAGS += -l $(shell dirname $(IMAGE).elf)/nemu-log.txt

CFLAGS += -DMAINARGS=\"$(mainargs)\"
CFLAGS += -I$(AM_HOME)/am/src/platform/nemu/include
.PHONY: $(AM_HOME)/am/src/platform/nemu/trm.c

NEMU_HOME := "/home/froster/code/Nemu-rust"

image: $(IMAGE).elf
	@$(OBJDUMP) -d $(IMAGE).elf > $(IMAGE).txt
	@echo + OBJCOPY "->" $(IMAGE_REL).bin
	@$(OBJCOPY) -S --set-section-flags .bss=alloc,contents -O binary $(IMAGE).elf $(IMAGE).bin

run: image
	cp $(IMAGE).bin "$(NEMU_HOME)/tests"
	cd $(NEMU_HOME) && cargo run --release --package nemu-rust --bin nemu-rust -- tests/$(NAME)-riscv64-nemu.bin --batch
	# $(MAKE) -C $(NEMU_HOME) ISA=$(ISA) run ARGS="$(NEMUFLAGS)" IMG=$(IMAGE).

run-step: image
	cp $(IMAGE).bin "$(NEMU_HOME)/tests"
	cd $(NEMU_HOME) && cargo run --release --package nemu-rust --bin nemu-rust -- tests/$(NAME)-riscv64-nemu.bin

run-diff: image
	cp $(IMAGE).bin "$(NEMU_HOME)/tests"
	cd $(NEMU_HOME) && cargo run --release --package nemu-rust --bin nemu-rust -- tests/$(NAME)-riscv64-nemu.bin --difftest

objdump: image
	$(OBJDUMP) -d $(IMAGE).elf

gdb: image
	$(MAKE) -C $(NEMU_HOME) ISA=$(ISA) gdb ARGS="$(NEMUFLAGS)" IMG=$(IMAGE).bin
