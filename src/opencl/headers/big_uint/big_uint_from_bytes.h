#include "src/opencl/structs/structs.h"

Uint256 uint256_from_bytes(const unsigned char *input);

ulong ulong_from_bytes(const unsigned char *input);

void bytes_to_uint256(const unsigned char *input, Uint256 *result);

ulong bytes_to_ulong(const unsigned char *input);