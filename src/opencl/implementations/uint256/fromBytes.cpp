#include "src/opencl/headers/uint256/fromBytes.h"

#pragma inline
UInt256 uint256FromBytes(const unsigned char *input)
{
  return (UInt256){(((unsigned long)(input[0]) << 56) | ((unsigned long)(input[1]) << 48) |
                    ((unsigned long)(input[2]) << 40) | ((unsigned long)(input[3]) << 32) |
                    ((unsigned long)(input[4]) << 24) | ((unsigned long)(input[5]) << 16) |
                    ((unsigned long)(input[6]) << 8) | ((unsigned long)(input[7]))),
                   (((unsigned long)(input[8]) << 56) | ((unsigned long)(input[9]) << 48) |
                    ((unsigned long)(input[10]) << 40) | ((unsigned long)(input[11]) << 32) |
                    ((unsigned long)(input[12]) << 24) | ((unsigned long)(input[13]) << 16) |
                    ((unsigned long)(input[14]) << 8) | ((unsigned long)(input[15]))),
                   (((unsigned long)(input[16]) << 56) | ((unsigned long)(input[17]) << 48) |
                    ((unsigned long)(input[18]) << 40) | ((unsigned long)(input[19]) << 32) |
                    ((unsigned long)(input[20]) << 24) | ((unsigned long)(input[21]) << 16) |
                    ((unsigned long)(input[22]) << 8) | ((unsigned long)(input[23]))),
                   (((unsigned long)(input[24]) << 56) | ((unsigned long)(input[25]) << 48) |
                    ((unsigned long)(input[26]) << 40) | ((unsigned long)(input[27]) << 32) |
                    ((unsigned long)(input[28]) << 24) | ((unsigned long)(input[29]) << 16) |
                    ((unsigned long)(input[30]) << 8) | ((unsigned long)(input[31])))};
};