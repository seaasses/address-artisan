#include "src/opencl/headers/hash/hash160.h"
#include "src/opencl/headers/hash/hash_common.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

#define HASH160_BSIG0(x) (ROTR(x, 2) ^ ROTR(x, 13) ^ ROTR(x, 22))
#define HASH160_BSIG1(x) (ROTR(x, 6) ^ ROTR(x, 11) ^ ROTR(x, 25))
#define HASH160_SSIG0(x) (ROTR(x, 7) ^ ROTR(x, 18) ^ SHR(x, 3))
#define HASH160_SSIG1(x) (ROTR(x, 17) ^ ROTR(x, 19) ^ SHR(x, 10))

#define HASH160_F(x, y, z) ((x) ^ (y) ^ (z))
#define HASH160_G(x, y, z) (((x) & (y)) | ((~(x)) & (z)))
#define HASH160_H(x, y, z) (((x) | (~(y))) ^ (z))
#define HASH160_I(x, y, z) (((x) & (z)) | ((y) & (~(z))))
#define HASH160_J(x, y, z) ((x) ^ ((y) | (~(z))))
#define HASH160_ROL(x, n) (((x) << (n)) | ((x) >> (32 - (n))))

inline void hash160_33(const uchar *restrict input, uchar *restrict output)
{
    unsigned char padded[64];

    uint W[64];

    uint H_sha[8] = {
        0x6a09e667, 0xbb67ae85,
        0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c,
        0x1f83d9ab, 0x5be0cd19};

#pragma unroll
    for (unsigned int i = 0; i < 33; i++)
    {
        padded[i] = input[i];
    }

    padded[33] = 0x80;

#pragma unroll
    for (uint i = 34; i < 62; i++)
    {
        padded[i] = 0x00;
    }

    padded[62] = 0x01; // 264 bits = 0x0108
    padded[63] = 0x08;

    uint a, b, c, d, e, f, g, h;
    uint T1, T2;
    unsigned int t;

#pragma unroll
    for (t = 0; t < 16; t++)
    {
        W[t] = UINT_FROM_BYTES_BE(padded + (t << 2));
    }

#pragma unroll
    for (t = 16; t < 64; t++)
    {
        W[t] = HASH160_SSIG1(W[t - 2]) + W[t - 7] + HASH160_SSIG0(W[t - 15]) + W[t - 16];
    }

    a = H_sha[0];
    b = H_sha[1];
    c = H_sha[2];
    d = H_sha[3];
    e = H_sha[4];
    f = H_sha[5];
    g = H_sha[6];
    h = H_sha[7];

    for (t = 0; t < 64; t++)
    {
        T1 = h + HASH160_BSIG1(e) + CH(e, f, g) + K_SHA256[t] + W[t];
        T2 = HASH160_BSIG0(a) + MAJ(a, b, c);
        h = g;
        g = f;
        f = e;
        e = d + T1;
        d = c;
        c = b;
        b = a;
        a = T1 + T2;
    }

    H_sha[0] += a;
    H_sha[1] += b;
    H_sha[2] += c;
    H_sha[3] += d;
    H_sha[4] += e;
    H_sha[5] += f;
    H_sha[6] += g;
    H_sha[7] += h;

    for (unsigned int i = 0; i < 8; i++)
    {
        UINT_TO_BYTES_BE(H_sha[i], padded + (i << 2));
    }

    // ====== RIPEMD160 ======
    uint H_ripemd[5] = {
        RIPEMD160_H0,
        RIPEMD160_H1,
        RIPEMD160_H2,
        RIPEMD160_H3,
        RIPEMD160_H4};

    padded[32] = 0x80;

#pragma unroll
    for (uint i = 33; i < 56; i++)
    {
        padded[i] = 0x00;
    }

    padded[56] = 0x00;
    padded[57] = 0x01;
    padded[58] = 0x00;
    padded[59] = 0x00;
    padded[60] = 0x00;
    padded[61] = 0x00;
    padded[62] = 0x00;
    padded[63] = 0x00;

    uint AL, BL, CL, DL, EL;
    uint AR, BR, CR, DR, ER;
    uint T;
    uint j;

#pragma unroll
    for (j = 0; j < 16; j++)
    {
        W[j] = UINT_FROM_BYTES_LE(padded + (j << 2));
    }

    AL = AR = H_ripemd[0];
    BL = BR = H_ripemd[1];
    CL = CR = H_ripemd[2];
    DL = DR = H_ripemd[3];
    EL = ER = H_ripemd[4];

    for (j = 0; j < 80; j++)
    {
        // Left line
        if (j < 16)
            T = AL + HASH160_F(BL, CL, DL) + W[r_L[j]] + K_L[0];
        else if (j < 32)
            T = AL + HASH160_G(BL, CL, DL) + W[r_L[j]] + K_L[1];
        else if (j < 48)
            T = AL + HASH160_H(BL, CL, DL) + W[r_L[j]] + K_L[2];
        else if (j < 64)
            T = AL + HASH160_I(BL, CL, DL) + W[r_L[j]] + K_L[3];
        else
            T = AL + HASH160_J(BL, CL, DL) + W[r_L[j]] + K_L[4];

        T = HASH160_ROL(T, s_L[j]) + EL;
        AL = EL;
        EL = DL;
        DL = HASH160_ROL(CL, 10);
        CL = BL;
        BL = T;

        // Right line
        if (j < 16)
            T = AR + HASH160_J(BR, CR, DR) + W[r_R[j]] + K_R[0];
        else if (j < 32)
            T = AR + HASH160_I(BR, CR, DR) + W[r_R[j]] + K_R[1];
        else if (j < 48)
            T = AR + HASH160_H(BR, CR, DR) + W[r_R[j]] + K_R[2];
        else if (j < 64)
            T = AR + HASH160_G(BR, CR, DR) + W[r_R[j]] + K_R[3];
        else
            T = AR + HASH160_F(BR, CR, DR) + W[r_R[j]] + K_R[4];

        T = HASH160_ROL(T, s_R[j]) + ER;
        AR = ER;
        ER = DR;
        DR = HASH160_ROL(CR, 10);
        CR = BR;
        BR = T;
    }

    T = H_ripemd[1] + CL + DR;
    H_ripemd[1] = H_ripemd[2] + DL + ER;
    H_ripemd[2] = H_ripemd[3] + EL + AR;
    H_ripemd[3] = H_ripemd[4] + AL + BR;
    H_ripemd[4] = H_ripemd[0] + BL + CR;
    H_ripemd[0] = T;

#pragma unroll
    for (uint i = 0; i < 5; i++)
    {
        UINT_TO_BYTES_LE(H_ripemd[i], output + (i << 2));
    }
}