#ifndef DINAH_H
#define DINAH_H

/* DINAH minimal scalar C ABI. No ownership or pointer-based API is exposed. */
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Returns ABI version 1. Call before using other dinah_abi_* functions. */
uint32_t dinah_abi_version(void);
/* Bit 0 means scalar metadata queries are available; remaining bits are reserved. */
uint64_t dinah_abi_features(void);
/* Returns 1 for Unix, 2 for Windows, or 0 for other targets. */
uint32_t dinah_abi_platform_family(void);

#ifdef __cplusplus
}
#endif
#endif /* DINAH_H */
