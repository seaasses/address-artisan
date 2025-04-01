#include "src/opencl/headers/uint256/toBytes.h"

#pragma inline
void uint256ToBytes(const UInt256 a, unsigned char *result)
{
  result[0] = a.limbs[0] >> 56;
  result[1] = a.limbs[0] >> 48;
  result[2] = a.limbs[0] >> 40;
  result[3] = a.limbs[0] >> 32;
  result[4] = a.limbs[0] >> 24;
  result[5] = a.limbs[0] >> 16;
  result[6] = a.limbs[0] >> 8;
  result[7] = a.limbs[0];

  result[8] = a.limbs[1] >> 56;
  result[9] = a.limbs[1] >> 48;
  result[10] = a.limbs[1] >> 40;
  result[11] = a.limbs[1] >> 32;
  result[12] = a.limbs[1] >> 24;
  result[13] = a.limbs[1] >> 16;
  result[14] = a.limbs[1] >> 8;
  result[15] = a.limbs[1];

  result[16] = a.limbs[2] >> 56;
  result[17] = a.limbs[2] >> 48;
  result[18] = a.limbs[2] >> 40;
  result[19] = a.limbs[2] >> 32;
  result[20] = a.limbs[2] >> 24;
  result[21] = a.limbs[2] >> 16;
  result[22] = a.limbs[2] >> 8;
  result[23] = a.limbs[2];

  result[24] = a.limbs[3] >> 56;
  result[25] = a.limbs[3] >> 48;
  result[26] = a.limbs[3] >> 40;
  result[27] = a.limbs[3] >> 32;
  result[28] = a.limbs[3] >> 24;
  result[29] = a.limbs[3] >> 16;
  result[30] = a.limbs[3] >> 8;
  result[31] = a.limbs[3];
}