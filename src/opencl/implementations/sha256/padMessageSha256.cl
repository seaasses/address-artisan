void padMessageSha256(uchar *message, ulong messageLength,
                      uint *paddedMessage) {
  // TODO: do directly with the paddedMessage

  uchar p[60];

  for (uint i = 0; i < messageLength; ++i) {
    p[i] = message[i];
  }

  p[messageLength] = 0x80;

#pragma unroll
  for (uchar i = messageLength + 1; i < 60; ++i) {
    p[i] = 0;
  }

#pragma unroll
  for (uchar i = 0; i < 15; i += 1) {
    paddedMessage[i] = (((uint)p[i << 2]) << 24) |
                       (((uint)p[(i << 2) + 1]) << 16) |
                       (((uint)p[(i << 2) + 2]) << 8) | ((uint)p[(i << 2) + 3]);
  }

  // 64 bits for the message length
  paddedMessage[15] = messageLength << 3;
}