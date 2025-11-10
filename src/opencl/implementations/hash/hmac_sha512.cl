#include "src/opencl/headers/hash/hmac_sha512.cl.h"
#include "src/opencl/headers/hash/sha512.cl.h"

inline void hmac_sha512_key32_msg37(const uchar *restrict key, const uchar *restrict message, uchar *restrict hash)
{
    uchar inner_message[SHA512_165_BYTES_MESSAGE_SIZE]; // 128 + 37 = 165
    uchar outer_message[SHA512_192_BYTES_MESSAGE_SIZE]; // 128 + 64 = 192

    // TODO: test if unrolling is faster
    for (uchar i = 0; i < HMAC_SHA512_KEY_SIZE; i++)
    {
        inner_message[i] = key[i] ^ HMAC_IPAD;
        outer_message[i] = key[i] ^ HMAC_OPAD;
    }

    for (uchar i = HMAC_SHA512_KEY_SIZE; i < SHA512_BLOCK_SIZE; i++)
    {
        inner_message[i] = HMAC_IPAD; // 0x00 ^ HMAC_IPAD
        outer_message[i] = HMAC_OPAD; // 0x00 ^ HMAC_OPAD
    }

    for (uchar i = 0; i < HMAC_SHA512_MESSAGE_SIZE; i++)
    {
        inner_message[SHA512_BLOCK_SIZE + i] = message[i];
    }

    sha512_165_bytes(inner_message, &outer_message[SHA512_BLOCK_SIZE]);

    sha512_192_bytes(outer_message, hash);
}
