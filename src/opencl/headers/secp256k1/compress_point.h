#include "src/opencl/structs/structs.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

#define COMPRESS_POINT(point, output)                                                  \
  do                                                                                   \
  {                                                                                    \
    (output)[0] = (uchar)(0x02 | (((uchar)((point).y.limbs[3])) & 1)); \
    uint256_to_bytes((point).x, &(output)[1]);                                         \
  } while (0)
