void padMessageSha256(unsigned char *message, unsigned long messageLength,
                      unsigned int *paddedMessage)
{
  // TODO: do directly with the paddedMessage

  unsigned char p[60];

  for (unsigned int i = 0; i < messageLength; ++i)
  {
    p[i] = message[i];
  }

  p[messageLength] = 0x80;

#pragma unroll
  for (unsigned char i = messageLength + 1; i < 60; ++i)
  {
    p[i] = 0;
  }

#pragma unroll
  for (unsigned char i = 0; i < 15; i += 1)
  {
    paddedMessage[i] = (((unsigned int)p[i << 2]) << 24) |
                       (((unsigned int)p[(i << 2) + 1]) << 16) |
                       (((unsigned int)p[(i << 2) + 2]) << 8) | ((unsigned int)p[(i << 2) + 3]);
  }

  // 64 bits for the message length
  paddedMessage[15] = messageLength << 3;
}