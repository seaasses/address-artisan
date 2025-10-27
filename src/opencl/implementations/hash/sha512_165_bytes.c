#include "src/opencl/headers/hash/sha512_165_bytes.h"

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

__constant ulong K[80] = {
    0x428a2f98d728ae22ULL, 0x7137449123ef65cdULL, 0xb5c0fbcfec4d3b2fULL, 0xe9b5dba58189dbbcULL,
    0x3956c25bf348b538ULL, 0x59f111f1b605d019ULL, 0x923f82a4af194f9bULL, 0xab1c5ed5da6d8118ULL,
    0xd807aa98a3030242ULL, 0x12835b0145706fbeULL, 0x243185be4ee4b28cULL, 0x550c7dc3d5ffb4e2ULL,
    0x72be5d74f27b896fULL, 0x80deb1fe3b1696b1ULL, 0x9bdc06a725c71235ULL, 0xc19bf174cf692694ULL,
    0xe49b69c19ef14ad2ULL, 0xefbe4786384f25e3ULL, 0x0fc19dc68b8cd5b5ULL, 0x240ca1cc77ac9c65ULL,
    0x2de92c6f592b0275ULL, 0x4a7484aa6ea6e483ULL, 0x5cb0a9dcbd41fbd4ULL, 0x76f988da831153b5ULL,
    0x983e5152ee66dfabULL, 0xa831c66d2db43210ULL, 0xb00327c898fb213fULL, 0xbf597fc7beef0ee4ULL,
    0xc6e00bf33da88fc2ULL, 0xd5a79147930aa725ULL, 0x06ca6351e003826fULL, 0x142929670a0e6e70ULL,
    0x27b70a8546d22ffcULL, 0x2e1b21385c26c926ULL, 0x4d2c6dfc5ac42aedULL, 0x53380d139d95b3dfULL,
    0x650a73548baf63deULL, 0x766a0abb3c77b2a8ULL, 0x81c2c92e47edaee6ULL, 0x92722c851482353bULL,
    0xa2bfe8a14cf10364ULL, 0xa81a664bbc423001ULL, 0xc24b8b70d0f89791ULL, 0xc76c51a30654be30ULL,
    0xd192e819d6ef5218ULL, 0xd69906245565a910ULL, 0xf40e35855771202aULL, 0x106aa07032bbd1b8ULL,
    0x19a4c116b8d2d0c8ULL, 0x1e376c085141ab53ULL, 0x2748774cdf8eeb99ULL, 0x34b0bcb5e19b48a8ULL,
    0x391c0cb3c5c95a63ULL, 0x4ed8aa4ae3418acbULL, 0x5b9cca4f7763e373ULL, 0x682e6ff3d6b2b8a3ULL,
    0x748f82ee5defb2fcULL, 0x78a5636f43172f60ULL, 0x84c87814a1f0ab72ULL, 0x8cc702081a6439ecULL,
    0x90befffa23631e28ULL, 0xa4506cebde82bde9ULL, 0xbef9a3f7b2c67915ULL, 0xc67178f2e372532bULL,
    0xca273eceea26619cULL, 0xd186b8c721c0c207ULL, 0xeada7dd6cde0eb1eULL, 0xf57d4f7fee6ed178ULL,
    0x06f067aa72176fbaULL, 0x0a637dc5a2c898a6ULL, 0x113f9804bef90daeULL, 0x1b710b35131c471bULL,
    0x28db77f523047d84ULL, 0x32caab7b40c72493ULL, 0x3c9ebe0a15c9bebcULL, 0x431d67c49c100d4cULL,
    0x4cc5d4becb3e42b6ULL, 0x597f299cfc657e2aULL, 0x5fcb6fab3ad6faecULL, 0x6c44198c4a475817ULL};

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
