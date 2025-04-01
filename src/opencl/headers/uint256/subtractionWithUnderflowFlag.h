#include "src/opencl/structs/uint256.h"

void uint256SubtractionWithUnderflowFlag(const UInt256 *a, const UInt256 *b,
                                         UInt256 *result, bool *underflowFlag);