__kernel void secp256k1Operations(__global uchar *x1, __global uchar *y1,
                                  __global uchar *x2, __global uchar *y2,
                                  __global uchar *scalar, uchar operation,
                                  __global uchar *resultX,
                                  __global uchar *resultY) {

  // THIS DOES NOT NEED TO BE FAST, IT IS ONLY USED FOR TESTING
  ulong index = get_global_id(0);

  uchar localX1[32];
  uchar localY1[32];
  uchar localX2[32];
  uchar localY2[32];
  uchar localResultX[32];
  uchar localResultY[32];
  uchar localScalarBytes[32];

#pragma unroll
  for (uchar i = 0; i < 32; i++) {
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
  if (operation == 0) {
    localResultPoint = sumPoints(x, y);
  } else if (operation == 1) {
    localResultPoint = gTimesScalar(localScalar);
  } else if (operation == 2) {
    localResultPoint = doublePoint(x);
  }

  uint256ToBytes(localResultPoint.x, localResultX);
  uint256ToBytes(localResultPoint.y, localResultY);

#pragma unroll
  for (uchar i = 0; i < 32; i++) {
    resultX[index * 32 + i] = localResultX[i];
    resultY[index * 32 + i] = localResultY[i];
  }
}