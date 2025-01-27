#include <am.h>
#include <nemu.h>

void __am_timer_init() {
}

void __am_timer_uptime(AM_TIMER_UPTIME_T *uptime) {
  volatile uint64_t *addr = (volatile uint64_t *)TIMER_ADDR;
  uptime->us = *addr;
}

void __am_timer_rtc(AM_TIMER_RTC_T *rtc) {
  volatile int32_t *addr = (volatile int32_t *)RTC_ADDR;
  rtc->second = addr[0];
  rtc->minute = addr[1];
  rtc->hour   = addr[2];
  rtc->day    = addr[3];
  rtc->month  = addr[4];
  rtc->year   = addr[5];
}
