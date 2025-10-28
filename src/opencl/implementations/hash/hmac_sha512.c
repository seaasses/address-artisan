#include "src/opencl/headers/hash/hmac_sha512.h"
#include "src/opencl/headers/hash/sha512.h"

void hmac_sha512_key32_msg37(const unsigned char *restrict key, const unsigned char *restrict message, unsigned char *restrict hash)
{
    unsigned char inner_message[SHA512_165_BYTES_MESSAGE_SIZE]; // 128 + 37 = 165
    unsigned char outer_message[SHA512_192_BYTES_MESSAGE_SIZE]; // 128 + 64 = 192

    // TODO: test if unrolling is faster
    for (unsigned char i = 0; i < HMAC_SHA512_KEY_SIZE; i++)
    {
        inner_message[i] = key[i] ^ HMAC_IPAD;
        outer_message[i] = key[i] ^ HMAC_OPAD;
    }

    for (unsigned char i = HMAC_SHA512_KEY_SIZE; i < SHA512_BLOCK_SIZE; i++)
    {
        inner_message[i] = HMAC_IPAD; // 0x00 ^ HMAC_IPAD
        outer_message[i] = HMAC_OPAD; // 0x00 ^ HMAC_OPAD
    }

    for (unsigned char i = 0; i < HMAC_SHA512_MESSAGE_SIZE; i++)
    {
        inner_message[SHA512_BLOCK_SIZE + i] = message[i];
    }

    sha512_165_bytes(inner_message, &outer_message[SHA512_BLOCK_SIZE]);

    sha512_192_bytes(outer_message, hash);
}
