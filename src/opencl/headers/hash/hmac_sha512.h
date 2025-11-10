#include "src/opencl/structs/structs.h"
#include "src/opencl/headers/hash/sha512.h"

#define HMAC_SHA512_KEY_SIZE 32
#define HMAC_SHA512_MESSAGE_SIZE 37
#define HMAC_SHA512_HASH_SIZE 64
#define SHA512_BLOCK_SIZE 128

#define HMAC_IPAD 0x36
#define HMAC_OPAD 0x5c

void hmac_sha512_key32_msg37(const uchar *restrict key, const uchar *restrict message, uchar *restrict hash);
