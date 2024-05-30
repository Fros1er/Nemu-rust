#include <am.h>
#include <riscv/riscv.h>
#include <klib.h>

static Context* (*user_handler)(Event, Context*) = NULL;

Context* __am_irq_handle(Context *c) {
  // printf("Context:\n");
  // for (int i = 0; i < 32; i++) {
  //   printf("%d: %x\n", i, c->gpr[i]);
  // }
  // printf("mcause: %x, mstatus: %x, mepc: %x\n", c->mcause, c->mstatus, c->mepc);
  if (user_handler) {
    Event ev = {0};
    switch (c->mcause) {
      case 11: {
        ev.event = c->gpr[17] == -1 ? EVENT_YIELD : EVENT_SYSCALL; 
        c->mepc += 4;
        break;
      }
      default: {
        printf("Unknown Event: %d\n", c->mcause);
        ev.event = EVENT_ERROR; break;
      }
    }

    c = user_handler(ev, c);
    assert(c != NULL);
  }
  // printf("mcause: %x, mstatus: %x, mepc: %x\n", c->mcause, c->mstatus, c->mepc);

  return c;
}

extern void __am_asm_trap(void);

bool cte_init(Context*(*handler)(Event, Context*)) {
  // initialize exception entry
  asm volatile("csrw mtvec, %0" : : "r"(__am_asm_trap));

  // register event handler
  user_handler = handler;

  return true;
}

Context *kcontext(Area kstack, void (*entry)(void *), void *arg) {
  return NULL;
}

void yield() {
#ifdef __riscv_e
  asm volatile("li a5, -1; ecall");
#else
  asm volatile("li a7, -1; ecall");
#endif
}

bool ienabled() {
  return false;
}

void iset(bool enable) {
}
