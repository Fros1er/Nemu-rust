#include <am.h>
#include <nemu.h>

#include "klib.h"

#define SYNC_ADDR (VGACTL_ADDR + 4)

static AM_GPU_CONFIG_T cfg = {};

void __am_gpu_config(AM_GPU_CONFIG_T *cfg) {
  uint32_t vgactl = inl(VGACTL_ADDR);
  *cfg = (AM_GPU_CONFIG_T){.present = true,
                           .has_accel = false,
                           .width = vgactl & 0xffff,
                           .height = vgactl >> 16,
                           .vmemsz = 0};
}

void __am_gpu_init() { __am_gpu_config(&cfg); }


void __am_gpu_fbdraw(AM_GPU_FBDRAW_T *ctl) {
  if (ctl->pixels != NULL) {
    uint32_t *pixels = (uint32_t *)ctl->pixels;
    for (int i = 0; i < ctl->h; i++) {
      for (int j = 0; j < ctl->w; j++) {
        outl(FB_ADDR + 4 * (cfg.width * (ctl->y + i) + ctl->x + j), pixels[i * ctl->w + j]);
      }
    }
  }
  if (ctl->sync) {
    outl(SYNC_ADDR, 1);
  }
}

void __am_gpu_status(AM_GPU_STATUS_T *status) { status->ready = true; }
