#include "src/opencl/headers/uint256/fromBytes.h"
#include "src/opencl/headers/uint256/toBytes.h"
#include "src/opencl/headers/secp256k1/pointAddition.h"


__kernel void secp256k1PointAdditionKernel(
    __global unsigned char *x1,
    __global unsigned char *y1,
    __global unsigned char *x2,
    __global unsigned char *y2,
    __global unsigned char *xResult,
    __global unsigned char *yResult)
{

  unsigned char localX1[32];
  unsigned char localY1[32];
  unsigned char localX2[32];
  unsigned char localY2[32];
  unsigned char localXResult[32];
  unsigned char localYResult[32];

  for (unsigned char i = 0; i < 32; i++)
  {
    localX1[i] = x1[i];
    localY1[i] = y1[i];
    localX2[i] = x2[i];
    localY2[i] = y2[i];
  }

  Point point1 = (Point){.x = uint256FromBytes(localX1), .y = uint256FromBytes(localY1)};
  Point point2 = (Point){.x = uint256FromBytes(localX2), .y = uint256FromBytes(localY2)};
  Point resultPoint;

  pointAddition(&point1, &point2, &resultPoint);

  uint256ToBytes(resultPoint.x, localXResult);
  uint256ToBytes(resultPoint.y, localYResult);

  for (unsigned char i = 0; i < 32; i++)
  {
    xResult[i] = localXResult[i];
    yResult[i] = localYResult[i];
  }
}
