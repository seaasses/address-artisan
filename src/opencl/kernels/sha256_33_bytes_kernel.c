#include "src/opencl/headers/hash/sha256.h"

__kernel void sha256_33_bytes_kernel(
    __global const unsigned char *input_message,
    __global unsigned char *output_hash)
{
    const unsigned int gid = get_global_id(0);
    const unsigned int message_offset = gid * SHA256_33_BYTES_MESSAGE_SIZE;
    const unsigned int hash_offset = gid * SHA256_HASH_SIZE;

    unsigned char local_message[SHA256_33_BYTES_MESSAGE_SIZE];
    unsigned char local_hash[SHA256_HASH_SIZE];

    for (unsigned char i = 0; i < SHA256_33_BYTES_MESSAGE_SIZE; i++)
    {
        local_message[i] = input_message[message_offset + i];
    }

    sha256_33_bytes(local_message, local_hash);

    for (unsigned char i = 0; i < SHA256_HASH_SIZE; i++)
    {
        output_hash[hash_offset + i] = local_hash[i];
    }
}
