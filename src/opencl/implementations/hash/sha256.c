#include "src/opencl/headers/hash/sha256.h"
#include "src/opencl/headers/hash/hash_common.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

// SHA256-specific sigma functions
#define BSIG0(x) (ROTR(x, 2) ^ ROTR(x, 13) ^ ROTR(x, 22))
#define BSIG1(x) (ROTR(x, 6) ^ ROTR(x, 11) ^ ROTR(x, 25))

#define SSIG0(x) (ROTR(x, 7) ^ ROTR(x, 18) ^ SHR(x, 3))
#define SSIG1(x) (ROTR(x, 17) ^ ROTR(x, 19) ^ SHR(x, 10))

inline void sha256_process_block(const unsigned char *restrict block, uint *restrict H)
{
    uint W[64];
    uint a, b, c, d, e, f, g, h;
    uint T1, T2;
    unsigned int t;

#pragma unroll
    for (t = 0; t < 16; t++)
    {
        W[t] = UINT_FROM_BYTES_BE(block + (t << 2));
    }

#pragma unroll
    for (t = 16; t < 64; t++)
    {
        W[t] = SSIG1(W[t - 2]) + W[t - 7] + SSIG0(W[t - 15]) + W[t - 16];
    }

    a = H[0];
    b = H[1];
    c = H[2];
    d = H[3];
    e = H[4];
    f = H[5];
    g = H[6];
    h = H[7];

    // NO unroll - degrades performance
    for (t = 0; t < 64; t++)
    {
        T1 = h + BSIG1(e) + CH(e, f, g) + K_SHA256[t] + W[t];
        T2 = BSIG0(a) + MAJ(a, b, c);
        h = g;
        g = f;
        f = e;
        e = d + T1;
        d = c;
        c = b;
        b = a;
        a = T1 + T2;
    }

    H[0] += a;
    H[1] += b;
    H[2] += c;
    H[3] += d;
    H[4] += e;
    H[5] += f;
    H[6] += g;
    H[7] += h;
}

inline void sha256_33_bytes(const unsigned char *restrict message, unsigned char *restrict hash)
{
    uint H[8] = {
        0x6a09e667, 0xbb67ae85,
        0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c,
        0x1f83d9ab, 0x5be0cd19};

    unsigned char padded[64];

#pragma unroll
    for (unsigned int i = 0; i < 33; i++)
    {
        padded[i] = message[i];
    }

    padded[33] = 0x80;

    uint *padded_words = (uint *)(padded + 34);
#pragma unroll
    for (unsigned int i = 0; i < 5; i++)
    {
        padded_words[i] = 0x00000000;
    }

    padded[60] = 0x00;
    padded[61] = 0x00;
    padded[62] = 0x01;
    padded[63] = 0x08;

    sha256_process_block(padded, H);

    for (unsigned int i = 0; i < 8; i++)
    {
        UINT_TO_BYTES_BE(H[i], hash + (i << 2));
    }
}
