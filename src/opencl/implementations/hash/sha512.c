#include "src/opencl/headers/hash/sha512.h"

#define CH(x, y, z) (((x) & (y)) | ((~(x)) & (z)))
#define MAJ(x, y, z) (((x) & ((y) | (z))) | ((y) & (z)))

#define ROTR(x, n) (((x) >> (n)) | ((x) << (64 - (n))))
#define SHR(x, n) ((x) >> (n))

#define BSIG0(x) (ROTR(x, 28) ^ ROTR(x, 34) ^ ROTR(x, 39))
#define BSIG1(x) (ROTR(x, 14) ^ ROTR(x, 18) ^ ROTR(x, 41))

#define SSIG0(x) (ROTR(x, 1) ^ ROTR(x, 8) ^ SHR(x, 7))
#define SSIG1(x) (ROTR(x, 19) ^ ROTR(x, 61) ^ SHR(x, 6))

#define BYTES_TO_WORD(bytes, offset)        \
    (((ulong)(bytes)[(offset)] << 56) |     \
     ((ulong)(bytes)[(offset) + 1] << 48) | \
     ((ulong)(bytes)[(offset) + 2] << 40) | \
     ((ulong)(bytes)[(offset) + 3] << 32) | \
     ((ulong)(bytes)[(offset) + 4] << 24) | \
     ((ulong)(bytes)[(offset) + 5] << 16) | \
     ((ulong)(bytes)[(offset) + 6] << 8) |  \
     ((ulong)(bytes)[(offset) + 7]))

#define WORD_TO_BYTES(word, bytes, offset)      \
    do                                          \
    {                                           \
        (bytes)[(offset)] = ((word) >> 56);     \
        (bytes)[(offset) + 1] = ((word) >> 48); \
        (bytes)[(offset) + 2] = ((word) >> 40); \
        (bytes)[(offset) + 3] = ((word) >> 32); \
        (bytes)[(offset) + 4] = ((word) >> 24); \
        (bytes)[(offset) + 5] = ((word) >> 16); \
        (bytes)[(offset) + 6] = ((word) >> 8);  \
        (bytes)[(offset) + 7] = (word);         \
    } while (0)

inline void sha512_process_block(const unsigned char *block, ulong *H)
{
    ulong W[80];
    ulong a, b, c, d, e, f, g, h;
    ulong T1, T2;
    unsigned int t;

#pragma unroll
    for (t = 0; t < 16; t++)
    {
        W[t] = BYTES_TO_WORD(block, t << 3);
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

inline void sha512_165_bytes(const unsigned char *restrict message, unsigned char *restrict hash)
{
    ulong H[8] = {
        0x6a09e667f3bcc908ULL, 0xbb67ae8584caa73bULL,
        0x3c6ef372fe94f82bULL, 0xa54ff53a5f1d36f1ULL,
        0x510e527fade682d1ULL, 0x9b05688c2b3e6c1fULL,
        0x1f83d9abfb41bd6bULL, 0x5be0cd19137e2179ULL};

    unsigned char padded[256];

// Copy message (165 bytes)
#pragma unroll
    for (unsigned int i = 0; i < 165; i++)
    {
        padded[i] = message[i];
    }

    // Padding byte
    padded[165] = 0x80;

    // Write 11 x 64-bit words starting at 166, then overwrite only non-zero bytes
    ulong *padded_words = (ulong *)(padded + 166);
#pragma unroll
    for (unsigned int i = 0; i < 11; i++)
    {
        padded_words[i] = 0x0000000000000000ULL;
    }
    // Message length is 1320 bits (165 bytes) = 0x0528
    padded[254] = 0x05;
    padded[255] = 0x28;

    sha512_process_block(padded, H);
    sha512_process_block(padded + 128, H);

    // NO unroll - no performance benefit
    for (unsigned int i = 0; i < 8; i++)
    {
        WORD_TO_BYTES(H[i], hash, i << 3);
    }
}

inline void sha512_192_bytes(const unsigned char *restrict message, unsigned char *restrict hash)
{
    ulong H[8] = {
        0x6a09e667f3bcc908ULL, 0xbb67ae8584caa73bULL,
        0x3c6ef372fe94f82bULL, 0xa54ff53a5f1d36f1ULL,
        0x510e527fade682d1ULL, 0x9b05688c2b3e6c1fULL,
        0x1f83d9abfb41bd6bULL, 0x5be0cd19137e2179ULL};

    unsigned char padded[256];

// Copy message (192 bytes)
#pragma unroll
    for (unsigned int i = 0; i < 192; i++)
    {
        padded[i] = message[i];
    }

    // Padding byte

    // Write 8 x 64-bit words starting at 192, then overwrite only non-zero bytes
    ulong *padded_words = (ulong *)(padded + 192);
#pragma unroll
    for (unsigned int i = 0; i < 8; i++)
    {
        padded_words[i] = 0x0000000000000000ULL;
    }

    padded[192] = 0x80;

    // Message length is 1536 bits (192 bytes) = 0x0600
    padded[254] = 0x06;
    padded[255] = 0x00;

    sha512_process_block(padded, H);
    sha512_process_block(padded + 128, H);

    // NO unroll - no performance benefit
    for (unsigned int i = 0; i < 8; i++)
    {
        WORD_TO_BYTES(H[i], hash, i << 3);
    }
}
