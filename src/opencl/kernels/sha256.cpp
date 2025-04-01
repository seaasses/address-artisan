#include "src/opencl/headers/sha256/sha256.h"

__kernel void sha256_kernel(__global unsigned char *message, unsigned int messageLength, __global unsigned char *sha256Result)
{

  const unsigned long workerId = get_global_id(0);
  unsigned char localMessage[55];

  for (unsigned int i = 0; i < messageLength; i++)
  {
    localMessage[i] = message[i];
  }

  if (workerId > 0)
  {
    return;
  }

  unsigned char hashedMessage[32];

  sha256(localMessage, (unsigned long)messageLength, hashedMessage);

#pragma unroll
  for (unsigned char i = 0; i < 32; ++i)
  {
    sha256Result[i] = hashedMessage[i];
  }
}