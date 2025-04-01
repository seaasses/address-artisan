#include "src/opencl/headers/sha512/padMessageSha512.h"

void padMessageSha512(unsigned char *message, unsigned long messageLength,
                      unsigned long *paddedMessage)
{
  // TODO: do directly with the paddedMessage

  unsigned char p[128]; // 1024 bits

#pragma unroll
  for (unsigned int i = 0; i < messageLength; ++i)
  {
    p[i] = message[i];
  }

  p[messageLength] = 0x80;

  // complete with 0s until 112 bytes
#pragma unroll
  for (unsigned char i = messageLength + 1; i < 112; ++i)
  {
    p[i] = 0;
  }

#pragma unroll
  for (unsigned char i = 0; i < 14; ++i)
  {
    paddedMessage[i] = ((((unsigned long)p[i << 3]) << 56)) |
                       ((((unsigned long)p[(i << 3) + 1]) << 48)) |
                       ((((unsigned long)p[(i << 3) + 2]) << 40)) |
                       ((((unsigned long)p[(i << 3) + 3]) << 32)) |
                       ((((unsigned long)p[(i << 3) + 4]) << 24)) |
                       ((((unsigned long)p[(i << 3) + 5]) << 16)) |
                       ((((unsigned long)p[(i << 3) + 6]) << 8)) |
                       (((unsigned long)p[(i << 3) + 7]));
  }

  // 128 bits (2 words) for the message length
  paddedMessage[14] = 0;
  paddedMessage[15] = messageLength << 3;
}