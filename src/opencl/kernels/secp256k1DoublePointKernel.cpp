#include "src/opencl/headers/uint256/fromBytes.h"
#include "src/opencl/headers/uint256/toBytes.h"
#include "src/opencl/headers/secp256k1/doublePoint.h"


__kernel void secp256k1DoublePointKernel(
    __global unsigned char *x,
    __global unsigned char *y,
    __global unsigned char *xResult,
    __global unsigned char *yResult)
{

  unsigned char localX[32];
  unsigned char localY[32];
  unsigned char localXResult[32];
  unsigned char localYResult[32];

  for (unsigned char i = 0; i < 32; i++)
  {
    localX[i] = x[i];
    localY[i] = y[i];
  }


  Point localPoint = (Point){.x = uint256FromBytes(localX), .y = uint256FromBytes(localY)};
  Point resultPoint;

  doublePoint(&localPoint, &resultPoint);

  uint256ToBytes(resultPoint.x, localXResult);
  uint256ToBytes(resultPoint.y, localYResult);

  for (unsigned char i = 0; i < 32; i++)
  {
    xResult[i] = localXResult[i];
    yResult[i] = localYResult[i];
  }
}
