#include "src/opencl/headers/hash/sha512.h"

__kernel void sha512_165_bytes_kernel(
    __global const uchar *input_message,
    __global uchar *output_hash)
{
    const unsigned int gid = get_global_id(0);
    const unsigned int message_offset = gid * SHA512_165_BYTES_MESSAGE_SIZE;
    const unsigned int hash_offset = gid * SHA512_HASH_SIZE;

    uchar local_message[SHA512_165_BYTES_MESSAGE_SIZE];
    uchar local_hash[SHA512_HASH_SIZE];

    for (uchar i = 0; i < SHA512_165_BYTES_MESSAGE_SIZE; i++)
    {
        local_message[i] = input_message[message_offset + i];
    }

    sha512_165_bytes(local_message, local_hash);

    for (uchar i = 0; i < SHA512_HASH_SIZE; i++)
    {
        output_hash[hash_offset + i] = local_hash[i];
    }
}
