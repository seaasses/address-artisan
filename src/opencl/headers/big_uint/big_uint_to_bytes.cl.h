#include "src/opencl/structs/structs.cl.h"

void uint256_to_bytes(const Uint256 a, uchar *result);

void uint320_to_bytes(const Uint320 a, uchar *result);

void uint512_to_bytes(const Uint512 a, uchar *result);

// Macros that work with any address space (__constant, __global, __private, __local)
#define ULONG_TO_BYTES(value, result) \
    do { \
        (result)[0] = (value) >> 56; \
        (result)[1] = (value) >> 48; \
        (result)[2] = (value) >> 40; \
        (result)[3] = (value) >> 32; \
        (result)[4] = (value) >> 24; \
        (result)[5] = (value) >> 16; \
        (result)[6] = (value) >> 8; \
        (result)[7] = (value); \
    } while (0)

#define UINT_TO_BYTES_BE(value, result) \
    do { \
        (result)[0] = (value) >> 24; \
        (result)[1] = (value) >> 16; \
        (result)[2] = (value) >> 8; \
        (result)[3] = (value); \
    } while (0)

#define UINT_TO_BYTES_LE(value, result) \
    do { \
        (result)[0] = (value); \
        (result)[1] = (value) >> 8; \
        (result)[2] = (value) >> 16; \
        (result)[3] = (value) >> 24; \
    } while (0)