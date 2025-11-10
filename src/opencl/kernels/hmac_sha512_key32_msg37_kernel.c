#include "src/opencl/headers/hash/hmac_sha512.h"

__kernel void hmac_sha512_key32_msg37_kernel(
    __global const uchar *input_key,
    __global const uchar *input_message,
    __global uchar *output_hash)
{
    const unsigned int gid = get_global_id(0);
    const unsigned int key_offset = gid * HMAC_SHA512_KEY_SIZE;
    const unsigned int message_offset = gid * HMAC_SHA512_MESSAGE_SIZE;
    const unsigned int hash_offset = gid * HMAC_SHA512_HASH_SIZE;

    uchar local_key[HMAC_SHA512_KEY_SIZE];
    uchar local_message[HMAC_SHA512_MESSAGE_SIZE];
    uchar local_hash[HMAC_SHA512_HASH_SIZE];

    for (uchar i = 0; i < HMAC_SHA512_KEY_SIZE; i++)
    {
        local_key[i] = input_key[key_offset + i];
    }

    for (uchar i = 0; i < HMAC_SHA512_MESSAGE_SIZE; i++)
    {
        local_message[i] = input_message[message_offset + i];
    }

    hmac_sha512_key32_msg37(local_key, local_message, local_hash);

    for (uchar i = 0; i < HMAC_SHA512_HASH_SIZE; i++)
    {
        output_hash[hash_offset + i] = local_hash[i];
    }
}
