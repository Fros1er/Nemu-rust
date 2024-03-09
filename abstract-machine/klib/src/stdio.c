#include <am.h>
#include <klib.h>
#include <klib-macros.h>
#include <stdarg.h>
#include <stdbool.h>

#if !defined(__ISA_NATIVE__) || defined(__NATIVE_USE_KLIB__)

#define putstr_buf(s, buf) \
({ for (const char *p = s; *p; p++) putch_checked(*p, buf); })

int itoa(int value, char *dst, int radix, bool is_sign) {
    unsigned int v;
    char *t = dst;

    int sign = is_sign && (radix == 10 && value < 0);
    if (sign)
        v = -value;
    else
        v = (unsigned) value;

    while (v || t == dst) {
        int i = v % radix;
        v /= radix;
        if (i < 10) {
            *(t++) = '0' + i;
        } else {
            *(t++) = 'a' + i - 10;
        }
    }
    if (sign) {
        *(t++) = '-';
    }
    *t = '\0';
    int len = t - dst;
    int l = 0, r = len - 1;
    while (l < r) {
        char tmp = dst[l];
        dst[l++] = dst[r];
        dst[r--] = tmp;
    }
    return len + 1;
}

int atoi_without_blank(const char **ptr) {
    int x = 0;
    const char *nptr = *ptr;
    while (*nptr >= '0' && *nptr <= '9') {
        x = x * 10 + *nptr - '0';
        nptr++;
    }
    *ptr = nptr;
    return x;
}

static char fmt_buf[128] = {};

#define fmt_number(radix, sign) \
    int i = va_arg(args, int); \
    itoa(i, fmt_buf, radix, sign); \
    putstr_buf(fmt_buf, out)

#define putch_checked(c, out) \
do { \
putch(c, out);                \
++res;                              \
if (n && res >= n) {return res;} \
} while(0)

static int
fmt_impl(char **out, int n, const char *fmt, void(*putch)(char, char **), va_list args) {
    int res = 0;
    while (*fmt != '\0') {
        char c = *(fmt++);
        switch (c) {
            case '%': {
                const char *t = fmt;
                // flags
                bool sharp = false;
                if (*t == '#') {
                    sharp = true;
                    t++;
                }
                // width
//                int padding = atoi_without_blank(&t);
                // precision
//                int precision = -1;
//                if (*t == '.') {
//                    t++;
//                    precision = atoi_without_blank(&t);
//                }
                // length Not implemented

                // specifier
                switch (*(t++)) {
                    case 'i':
                    case 'd': { // signed decimal
                        fmt_number(10, true);
                        break;
                    }
                    case 'u': {
                        fmt_number(10, false);
                        break;
                    }
                    case 'o': {
                        if (sharp) {
                            putch_checked('0', out);
                        }
                        fmt_number(8, false);
                        break;
                    }
                    case 'x': {
                        if (sharp) putstr_buf("0x", out);
                        fmt_number(16, false);
                        break;
                    }
                    case 'X': {
                        if (sharp) putstr_buf("0X", out);
                        fmt_number(16, false);
                        break;
                    }
                    case 'c': {
                        int i = va_arg(args,
                        int);
                        putch_checked((char) i, out);
                        break;
                    }
                    case 's': {
                        const char *s = va_arg(args,
                        const char *);
                        putstr_buf(s, out);
                        break;
                    }
                    default: {
                        putch_checked('%', out);
                        goto UNKNOWN_SPEC;
                    }
                }
                fmt = t;

                UNKNOWN_SPEC:
                break;
            }
            default: {
                putch_checked(c, out);
            }
        }
    }
    if (out != NULL) {
        putch_checked('\0', out);
    }
    return res;
}

void stdio_putch(char c, char **_buf) {
    putch(c);
}

int printf(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    int res = fmt_impl(NULL, 0, fmt, stdio_putch, args);
    va_end(args);
    return res;
}

void buf_putch(char c, char **buf) {
    **buf = c;
    (*buf) += 1;
}

int vsprintf(char *out, const char *fmt, va_list args) {
    char *tmp = out;
    return fmt_impl(&tmp, 0, fmt, buf_putch, args);
}

int sprintf(char *out, const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    int res = vsprintf(out, fmt, args);
    va_end(args);
    return res;
}

int snprintf(char *out, size_t n, const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    int res = vsnprintf(out, n, fmt, args);
    va_end(args);
    return res;
}

int vsnprintf(char *out, size_t n, const char *fmt, va_list args) {
    char *tmp = out;
    return fmt_impl(&tmp, n, fmt, buf_putch, args);
}

#endif
