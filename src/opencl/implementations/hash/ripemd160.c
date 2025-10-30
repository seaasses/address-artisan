#include "src/opencl/headers/hash/ripemd160.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

// RIPEMD160-specific functions
#define F(x, y, z) ((x) ^ (y) ^ (z))
#define G(x, y, z) (((x) & (y)) | ((~(x)) & (z)))
#define H(x, y, z) (((x) | (~(y))) ^ (z))
#define I(x, y, z) (((x) & (z)) | ((y) & (~(z))))
#define J(x, y, z) ((x) ^ ((y) | (~(z))))

// Rotate left for RIPEMD160 (uint/32-bit)
#define ROL(x, n) (((x) << (n)) | ((x) >> (32 - (n))))

inline void ripemd160_process_block(const uchar *restrict block, uint *restrict H)
{
    uint X[16];
    uint AL, BL, CL, DL, EL; // Left line
    uint AR, BR, CR, DR, ER; // Right line
    uint T;
    uint j;

    // Load message into X array (little-endian)
#pragma unroll
    for (j = 0; j < 16; j++)
    {
        X[j] = UINT_FROM_BYTES_LE(block + (j << 2));
    }

    // Initialize working variables
    AL = AR = H[0];
    BL = BR = H[1];
    CL = CR = H[2];
    DL = DR = H[3];
    EL = ER = H[4];

    for (j = 0; j < 80; j++)
    {
        // Left line
        if (j < 16)
            T = AL + F(BL, CL, DL) + X[r_L[j]] + K_L[0];
        else if (j < 32)
            T = AL + G(BL, CL, DL) + X[r_L[j]] + K_L[1];
        else if (j < 48)
            T = AL + H(BL, CL, DL) + X[r_L[j]] + K_L[2];
        else if (j < 64)
            T = AL + I(BL, CL, DL) + X[r_L[j]] + K_L[3];
        else
            T = AL + J(BL, CL, DL) + X[r_L[j]] + K_L[4];

        T = ROL(T, s_L[j]) + EL;
        AL = EL;
        EL = DL;
        DL = ROL(CL, 10);
        CL = BL;
        BL = T;

        // Right line
        if (j < 16)
            T = AR + J(BR, CR, DR) + X[r_R[j]] + K_R[0];
        else if (j < 32)
            T = AR + I(BR, CR, DR) + X[r_R[j]] + K_R[1];
        else if (j < 48)
            T = AR + H(BR, CR, DR) + X[r_R[j]] + K_R[2];
        else if (j < 64)
            T = AR + G(BR, CR, DR) + X[r_R[j]] + K_R[3];
        else
            T = AR + F(BR, CR, DR) + X[r_R[j]] + K_R[4];

        T = ROL(T, s_R[j]) + ER;
        AR = ER;
        ER = DR;
        DR = ROL(CR, 10);
        CR = BR;
        BR = T;
    }

    // Combine results
    T = H[1] + CL + DR;
    H[1] = H[2] + DL + ER;
    H[2] = H[3] + EL + AR;
    H[3] = H[4] + AL + BR;
    H[4] = H[0] + BL + CR;
    H[0] = T;
}

inline void ripemd160_32_bytes(const uchar *restrict message, uchar *restrict hash)
{
    uint H[5] = {
        RIPEMD160_H0,
        RIPEMD160_H1,
        RIPEMD160_H2,
        RIPEMD160_H3,
        RIPEMD160_H4};

    uchar padded[64];

// Copy message (32 bytes)
#pragma unroll
    for (uint i = 0; i < 32; i++)
    {
        padded[i] = message[i];
    }

    padded[32] = 0x80;

    // Zerar bytes [33-55]
#pragma unroll
    for (uint i = 33; i < 56; i++)
    {
        padded[i] = 0x00;
    }

    // Tamanho em bits (32 bytes * 8 = 256 bits = 0x0100) em little-endian
    padded[56] = 0x00;
    padded[57] = 0x01;
    padded[58] = 0x00;
    padded[59] = 0x00;
    padded[60] = 0x00;
    padded[61] = 0x00;
    padded[62] = 0x00;
    padded[63] = 0x00;

    ripemd160_process_block(padded, H);

#pragma unroll
    for (uint i = 0; i < 5; i++)
    {
        UINT_TO_BYTES_LE(H[i], hash + (i << 2));
    }
}
