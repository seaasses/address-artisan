#include "src/opencl/structs/structs.h"
#include "src/opencl/headers/uint256/fromBytes.h"
#include "src/opencl/headers/uint256/toBytes.h"
#include "src/opencl/headers/secp256k1/sumPoints.h"
#include "src/opencl/headers/secp256k1/doublePoint.h"

__kernel void secp256k1Operations(
    __global unsigned char *x1, __global unsigned char *y1,
    __global unsigned char *x2, __global unsigned char *y2,
    __global unsigned char *scalar, unsigned char operation,
    __global unsigned char *resultX,
    __global unsigned char *resultY)
{

  // THIS DOES NOT NEED TO BE FAST, IT IS ONLY USED FOR TESTING
  unsigned long index = get_global_id(0);

  unsigned char localX1[32];
  unsigned char localY1[32];
  unsigned char localX2[32];
  unsigned char localY2[32];
  unsigned char localResultX[32];
  unsigned char localResultY[32];
  unsigned char localScalarBytes[32];

#pragma unroll
  for (unsigned char i = 0; i < 32; i++)
  {
    localX1[i] = x1[index * 32 + i];
    localY1[i] = y1[index * 32 + i];
    localX2[i] = x2[index * 32 + i];
    localY2[i] = y2[index * 32 + i];
    localScalarBytes[i] = scalar[index * 32 + i];
  }

  Point x = {.x = uint256FromBytes(localX1), .y = uint256FromBytes(localY1)};
  Point y = {.x = uint256FromBytes(localX2), .y = uint256FromBytes(localY2)};
  Point localResultPoint;
  UInt256 localScalar = uint256FromBytes(localScalarBytes);

  //////////////////////////////////////
  if (operation == 0)
  {
    localResultPoint = sumPoints(x, y);
  }
  else if (operation == 1)
  {
    localResultPoint = doublePoint(x);
  }
  // else if (operation == 1)
  // {
  //   localResultPoint = gTimesScalar(localScalar);
  // }

  uint256ToBytes(localResultPoint.x, localResultX);
  uint256ToBytes(localResultPoint.y, localResultY);

#pragma unroll
  for (unsigned char i = 0; i < 32; i++)
  {
    resultX[index * 32 + i] = localResultX[i];
    resultY[index * 32 + i] = localResultY[i];
  }
}