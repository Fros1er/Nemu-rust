#include <am.h>
#include <nemu.h>

#define KEYDOWN_MASK 0x8000

void __am_input_init() {
}

void __am_input_config(AM_INPUT_CONFIG_T *cfg) {
    cfg->present = true;
}

void __am_input_keybrd(AM_INPUT_KEYBRD_T *kbd) {
    uint32_t info = inl(KBD_ADDR);
    kbd->keydown = (info & KEYDOWN_MASK) != 0;
    kbd->keycode = info & (0xffffffff ^ KEYDOWN_MASK);
    if (info != 0) {
        outl(KBD_ADDR + 4, 1);
    }
//    kbd->keycode = info & 0xff;
}
