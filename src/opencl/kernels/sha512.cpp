#include "src/opencl/headers/sha512/sha512.h"

__kernel void sha512_kernel(__global unsigned char *message, unsigned int messageLength,
                            __global unsigned char *sha512Result)
{

  const unsigned long workerId = get_global_id(0);
  unsigned char localMessage[111];

  for (uint i = 0; i < messageLength; i++)
  {
    localMessage[i] = message[i];
  }

  if (workerId > 0)
  {
    return;
  }

  unsigned char hashedMessage[64];

  sha512(localMessage, (unsigned long)messageLength, hashedMessage);

#pragma unroll
  for (unsigned char i = 0; i < 64; ++i)
  {
    sha512Result[i] = hashedMessage[i];
  }
}