#include "src/opencl/headers/hash/sha512.h"
#include "src/opencl/headers/hash/hash_common.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

// SHA512-specific sigma functions
#define BSIG0(x) (ROTR(x, 28) ^ ROTR(x, 34) ^ ROTR(x, 39))
#define BSIG1(x) (ROTR(x, 14) ^ ROTR(x, 18) ^ ROTR(x, 41))

#define SSIG0(x) (ROTR(x, 1) ^ ROTR(x, 8) ^ SHR(x, 7))
#define SSIG1(x) (ROTR(x, 19) ^ ROTR(x, 61) ^ SHR(x, 6))

inline void sha512_process_block(const uchar *restrict block, ulong *restrict H)
{
    ulong W[80];
    ulong a, b, c, d, e, f, g, h;
    ulong T1, T2;
    unsigned int t;

#pragma unroll
    for (t = 0; t < 16; t++)
    {
        W[t] = ULONG_FROM_BYTES(block + (t << 3));
    }

#pragma unroll
    for (t = 16; t < 80; t++)
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
    for (t = 0; t < 80; t++)
    {
        T1 = h + BSIG1(e) + CH(e, f, g) + K[t] + W[t];
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

inline void sha512_165_bytes(const uchar *restrict message, uchar *restrict hash)
{
    ulong H[8] = {
        0x6a09e667f3bcc908ULL, 0xbb67ae8584caa73bULL,
        0x3c6ef372fe94f82bULL, 0xa54ff53a5f1d36f1ULL,
        0x510e527fade682d1ULL, 0x9b05688c2b3e6c1fULL,
        0x1f83d9abfb41bd6bULL, 0x5be0cd19137e2179ULL};

    uchar padded[256];

// Copy message (165 bytes)
#pragma unroll
    for (unsigned int i = 0; i < 165; i++)
    {
        padded[i] = message[i];
    }

    // Padding byte
    padded[165] = 0x80;

#pragma unroll
    for (uint i = 166; i < 254; i++)
    {
        padded[i] = 0x00;
    }
    // Message length is 1320 bits (165 bytes) = 0x0528
    padded[254] = 0x05;
    padded[255] = 0x28;

    sha512_process_block(padded, H);
    sha512_process_block(padded + 128, H);

    // NO unroll - no performance benefit
    for (unsigned int i = 0; i < 8; i++)
    {
        ULONG_TO_BYTES(H[i], hash + (i << 3));
    }
}

inline void sha512_192_bytes(const uchar *restrict message, uchar *restrict hash)
{
    ulong H[8] = {
        0x6a09e667f3bcc908ULL, 0xbb67ae8584caa73bULL,
        0x3c6ef372fe94f82bULL, 0xa54ff53a5f1d36f1ULL,
        0x510e527fade682d1ULL, 0x9b05688c2b3e6c1fULL,
        0x1f83d9abfb41bd6bULL, 0x5be0cd19137e2179ULL};

    uchar padded[256];

// Copy message (192 bytes)
#pragma unroll
    for (unsigned int i = 0; i < 192; i++)
    {
        padded[i] = message[i];
    }

    // Padding byte
    padded[192] = 0x80;

#pragma unroll
    for (uint i = 193; i < 254; i++)
    {
        padded[i] = 0x00;
    }

    // Message length is 1536 bits (192 bytes) = 0x0600
    padded[254] = 0x06;
    padded[255] = 0x00;

    sha512_process_block(padded, H);
    sha512_process_block(padded + 128, H);

    // NO unroll - no performance benefit
    for (unsigned int i = 0; i < 8; i++)
    {
        ULONG_TO_BYTES(H[i], hash + (i << 3));
    }
}
