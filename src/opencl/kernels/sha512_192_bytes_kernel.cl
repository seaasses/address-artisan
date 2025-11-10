#include "src/opencl/headers/hash/sha512.h"

__kernel void sha512_192_bytes_kernel(
    __global const uchar *input_message,
    __global uchar *output_hash)
{
    const uint gid = get_global_id(0);
    const uint message_offset = gid * SHA512_192_BYTES_MESSAGE_SIZE;
    const uint hash_offset = gid * SHA512_HASH_SIZE;

    uchar local_message[SHA512_192_BYTES_MESSAGE_SIZE];
    uchar local_hash[SHA512_HASH_SIZE];

    for (uchar i = 0; i < SHA512_192_BYTES_MESSAGE_SIZE; i++)
    {
        local_message[i] = input_message[message_offset + i];
    }

    sha512_192_bytes(local_message, local_hash);

    for (uchar i = 0; i < SHA512_HASH_SIZE; i++)
    {
        output_hash[hash_offset + i] = local_hash[i];
    }
}
