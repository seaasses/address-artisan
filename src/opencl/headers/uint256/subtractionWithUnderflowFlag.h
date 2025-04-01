#include "src/opencl/structs/structs.h"

void uint256SubtractionWithUnderflowFlag(const UInt256 *a, const UInt256 *b,
                                         UInt256 *result, bool *underflowFlag);