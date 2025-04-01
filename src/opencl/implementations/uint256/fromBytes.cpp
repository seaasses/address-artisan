#include "src/opencl/headers/uint256/fromBytes.h"

#pragma inline
UInt256 uint256FromBytes(const unsigned char *input)
{
  return (UInt256){(((ulong)(input[0]) << 56) | ((ulong)(input[1]) << 48) |
                    ((ulong)(input[2]) << 40) | ((ulong)(input[3]) << 32) |
                    ((ulong)(input[4]) << 24) | ((ulong)(input[5]) << 16) |
                    ((ulong)(input[6]) << 8) | ((ulong)(input[7]))),
                   (((ulong)(input[8]) << 56) | ((ulong)(input[9]) << 48) |
                    ((ulong)(input[10]) << 40) | ((ulong)(input[11]) << 32) |
                    ((ulong)(input[12]) << 24) | ((ulong)(input[13]) << 16) |
                    ((ulong)(input[14]) << 8) | ((ulong)(input[15]))),
                   (((ulong)(input[16]) << 56) | ((ulong)(input[17]) << 48) |
                    ((ulong)(input[18]) << 40) | ((ulong)(input[19]) << 32) |
                    ((ulong)(input[20]) << 24) | ((ulong)(input[21]) << 16) |
                    ((ulong)(input[22]) << 8) | ((ulong)(input[23]))),
                   (((ulong)(input[24]) << 56) | ((ulong)(input[25]) << 48) |
                    ((ulong)(input[26]) << 40) | ((ulong)(input[27]) << 32) |
                    ((ulong)(input[28]) << 24) | ((ulong)(input[29]) << 16) |
                    ((ulong)(input[30]) << 8) | ((ulong)(input[31])))};
};