#include "src/opencl/structs/structs.h"

void uint256_subtraction(const Uint256 *a, const Uint256 *b, Uint256 *result);

void uint256_subtraction_with_underflow_flag(const Uint256 *a, const Uint256 *b,
                                             Uint256 *result, unsigned int *underflowFlag);