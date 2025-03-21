void padMessageSha512(uchar *message, ulong messageLength,
                      ulong *paddedMessage) {
  // TODO: do directly with the paddedMessage

  uchar p[128]; // 1024 bits

#pragma unroll
  for (uint i = 0; i < messageLength; ++i) {
    p[i] = message[i];
  }

  p[messageLength] = 0x80;

  // complete with 0s until 112 bytes
#pragma unroll
  for (uchar i = messageLength + 1; i < 112; ++i) {
    p[i] = 0;
  }

#pragma unroll
  for (uchar i = 0; i < 14; ++i) {
    paddedMessage[i] = ((((ulong)p[i << 3]) << 56)) |
                       ((((ulong)p[(i << 3) + 1]) << 48)) |
                       ((((ulong)p[(i << 3) + 2]) << 40)) |
                       ((((ulong)p[(i << 3) + 3]) << 32)) |
                       ((((ulong)p[(i << 3) + 4]) << 24)) |
                       ((((ulong)p[(i << 3) + 5]) << 16)) |
                       ((((ulong)p[(i << 3) + 6]) << 8)) |
                       (((ulong)p[(i << 3) + 7]));
  }

  // 128 bits (2 words) for the message length
  paddedMessage[14] = 0;
  paddedMessage[15] = messageLength << 3;
}