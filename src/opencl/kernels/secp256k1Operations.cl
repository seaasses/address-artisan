__kernel void secp256k1Operations(__global uchar *x1, __global uchar *y1,
                                  __global uchar *x2, __global uchar *y2,
                                  uchar operation, __global uchar *result_x,
                                  __global uchar *result_y) {

  // THIS DOES NOT NEED TO BE FAST, IT IS ONLY USED FOR TESTING
  ulong index = get_global_id(0);


  uchar local_x1[32];
  uchar local_y1[32];
  uchar local_x2[32];
  uchar local_y2[32];
  uchar local_result_x[32];
  uchar local_result_y[32];

#pragma unroll
  for (uchar i = 0; i < 32; i++) {
    local_x1[i] = x1[index * 32 + i];
    local_y1[i] = y1[index * 32 + i];
    local_x2[i] = x2[index * 32 + i];
    local_y2[i] = y2[index * 32 + i];
  }

  Point x = {.x = uint256FromBytes(local_x1), .y = uint256FromBytes(local_y1)};
  Point y = {.x = uint256FromBytes(local_x2), .y = uint256FromBytes(local_y2)};
  Point localResultPoint;

  //////////////////////////////////////
  if (operation == 0) {
    localResultPoint = sumPoints(x, y);
  }

  uint256ToBytes(localResultPoint.x, local_result_x);
  uint256ToBytes(localResultPoint.y, local_result_y);

#pragma unroll
  for (uchar i = 0; i < 32; i++) {
    result_x[index * 32 + i] = local_result_x[i];
    result_y[index * 32 + i] = local_result_y[i];
  }
}