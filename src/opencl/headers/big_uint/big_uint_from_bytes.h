#include "src/opencl/structs/structs.h"

Uint256 uint256_from_bytes(const unsigned char *restrict input);

ulong ulong_from_bytes(const unsigned char *restrict input);

void bytes_to_uint256(const unsigned char *restrict input, Uint256 *restrict result);

ulong bytes_to_ulong(const unsigned char *restrict input);