void padMessageSha512(uchar *message, ulong messageLength,
                      ulong *paddedMessage) {
  // TODO: do directly with the paddedMessage

  uchar p[128]; // 1024 bits

  for (uint i = 0; i < messageLength; ++i) {
    p[i] = message[i];
  }

  p[messageLength] = 0x80;

#pragma unroll
  // complete with 0s until 112 bytes
  for (uchar i = messageLength + 1; i < 112; ++i) {
    p[i] = 0;
  }

  uchar *paddedMessageBytes = (uchar *)paddedMessage;
  uchar *messageLengthBytes = (uchar *)&messageLength;

  if (isLittleEndian()) {
#pragma unroll
    for (uchar i = 0; i < 112; i += 8) {
      paddedMessageBytes[i] = p[i + 7];
      paddedMessageBytes[i + 1] = p[i + 6];
      paddedMessageBytes[i + 2] = p[i + 5];
      paddedMessageBytes[i + 3] = p[i + 4];
      paddedMessageBytes[i + 4] = p[i + 3];
      paddedMessageBytes[i + 5] = p[i + 2];
      paddedMessageBytes[i + 6] = p[i + 1];
      paddedMessageBytes[i + 7] = p[i];
    }
  } else {
#pragma unroll
    for (uchar i = 0; i < 112; ++i) {
      paddedMessageBytes[i] = p[i];
    }
  }

  // 128 bits (2 words) for the message length
  paddedMessage[14] = 0;
  paddedMessage[15] = messageLength << 3;
}