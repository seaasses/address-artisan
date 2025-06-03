#include "src/opencl/structs/structs.h"

void uint256_addition(const Uint256 *a, const Uint256 *b, Uint256 *result);

void uint256_addition_with_overflow_flag(const Uint256 *a, const Uint256 *b, Uint256 *result, unsigned int *overflowFlag);

void uint320_uint256_addition(const Uint320 *a, const Uint256 *b, Uint320 *result);

