#include "src/opencl/headers/hash/sha256.h"
#include "src/opencl/headers/hash/ripemd160.h"

#define HASH160_INPUT_SIZE 33
#define HASH160_OUTPUT_SIZE 20

#define HASH160_33(input, output)                        \
  do                                                     \
  {                                                      \
    unsigned char sha256_hash[SHA256_HASH_SIZE];         \
    sha256_33_bytes((input), sha256_hash);               \
    ripemd160_32_bytes(sha256_hash, (output));           \
  } while (0)
