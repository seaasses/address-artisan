#include "src/opencl/structs/structs.h"

#define SHA512_165_BYTES_MESSAGE_SIZE 165
#define SHA512_HASH_SIZE 64

void sha512_165_bytes(const unsigned char *restrict message, unsigned char *restrict hash);
