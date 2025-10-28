#include "src/opencl/headers/hash/ripemd160.h"

// RIPEMD160-specific functions
#define F(x, y, z) ((x) ^ (y) ^ (z))
#define G(x, y, z) (((x) & (y)) | ((~(x)) & (z)))
#define H(x, y, z) (((x) | (~(y))) ^ (z))
#define I(x, y, z) (((x) & (z)) | ((y) & (~(z))))
#define J(x, y, z) ((x) ^ ((y) | (~(z))))

// Rotate left for RIPEMD160 (uint/32-bit)
#define ROL(x, n) (((x) << (n)) | ((x) >> (32 - (n))))

// Little-endian conversion for RIPEMD160
#define BYTES_TO_WORD(bytes, offset)       \
    (((uint)(bytes)[(offset)]) |           \
     ((uint)(bytes)[(offset) + 1] << 8) |  \
     ((uint)(bytes)[(offset) + 2] << 16) | \
     ((uint)(bytes)[(offset) + 3] << 24))

#define WORD_TO_BYTES(word, bytes, offset)      \
    do                                          \
    {                                           \
        (bytes)[(offset)] = (word);             \
        (bytes)[(offset) + 1] = ((word) >> 8);  \
        (bytes)[(offset) + 2] = ((word) >> 16); \
        (bytes)[(offset) + 3] = ((word) >> 24); \
    } while (0)

inline void ripemd160_process_block(const unsigned char *restrict block, uint *restrict H)
{
    uint X[16];
    uint AL, BL, CL, DL, EL; // Left line
    uint AR, BR, CR, DR, ER; // Right line
    uint T;
    unsigned int j;

    // Load message into X array (little-endian)
#pragma unroll
    for (j = 0; j < 16; j++)
    {
        X[j] = BYTES_TO_WORD(block, j << 2);
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

inline void ripemd160_32_bytes(const unsigned char *restrict message, unsigned char *restrict hash)
{
    uint H[5] = {
        RIPEMD160_H0,
        RIPEMD160_H1,
        RIPEMD160_H2,
        RIPEMD160_H3,
        RIPEMD160_H4};

    unsigned char padded[64];

// Copy message (32 bytes)
#pragma unroll
    for (unsigned int i = 0; i < 32; i++)
    {
        padded[i] = message[i];
    }

    padded[32] = 0x80;

    uint *padded_words = (uint *)(padded + 33);
#pragma unroll
    for (unsigned int i = 0; i < 7; i++)
    {
        padded_words[i] = 0x00000000;
    }
    padded[57] = 0x01;

    ripemd160_process_block(padded, H);

#pragma unroll
    for (unsigned int i = 0; i < 5; i++)
    {
        WORD_TO_BYTES(H[i], hash, i << 2);
    }
}
