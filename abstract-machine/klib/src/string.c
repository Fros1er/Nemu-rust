#include <klib-macros.h>
#include <klib.h>
#include <stdint.h>

#if !defined(__ISA_NATIVE__) || defined(__NATIVE_USE_KLIB__)

size_t strlen(const char *s) {
  const char *end = s;
  while (*end != '\0') {
    ++end;
  }
  return end - s;
}

char *strcpy(char *dst, const char *src) {
  char *tmp = dst;
  while (*src != '\0') {
    *(tmp++) = *(src++);
  }
  *tmp = '\0';
  return dst;
}

char *strncpy(char *dst, const char *src, size_t n) {
  if (n == 0) {
    return dst;
  }
  char *tmp = dst;
  while (*src != '\0') {
    *(tmp++) = *(src++);
    if (--n == 0) {
      return dst;
    }
  }
  while (n-- != 0) {
    *(tmp++) = '\0';
  }
  return dst;
}

char *strcat(char *dst, const char *src) {
  char *tmp = dst;
  while (*tmp != '\0') {
    ++tmp;
  }
  strcpy(tmp, src);
  return dst;
}

int strcmp(const char *s1, const char *s2) {
  unsigned char c1, c2;

  do {
    c1 = *s1++;
    c2 = *s2++;
    if (c1 == '\0') return c1 - c2;
  } while (c1 == c2);

  return c1 - c2;
}

int strncmp(const char *s1, const char *s2, size_t n) {
  while (n-- > 0) {
    char u1 = *s1++;
    char u2 = *s2++;
    if (u1 != u2) return u1 - u2;
    if (u1 == '\0') return 0;
  }
  return 0;
}

void *memset(void *s, int c, size_t n) {
  uint8_t *p = s;
  while (n-- != 0) {
    *(p++) = c;
  }
  return s;
}

void *memmove(void *dst, const void *src, size_t n) {
  char *d = dst;
  const char *s = src;
  if (d < s)
    while (n--) *d++ = *s++;
  else {
    const char *lasts = s + (n - 1);
    char *lastd = d + (n - 1);
    while (n--) *lastd-- = *lasts--;
  }
  return dst;
}

void *memcpy(void *out, const void *in, size_t n) {
  const uint8_t *from = (uint8_t *)in;
  uint8_t *to = (uint8_t *)out;
  while (n-- != 0) {
    *(to++) = *(from++);
  }
  return out;
}

int memcmp(const void *str1, const void *str2, size_t n) {
  const uint8_t *s1 = str1;
  const uint8_t *s2 = str2;

  while (n-- > 0) {
    if (*s1++ != *s2++) {
      return s1[-1] < s2[-1] ? -1 : 1;
    }
  }
  return 0;
}

#endif
