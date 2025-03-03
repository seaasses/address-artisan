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

  uchar *paddedMessageBytes = (uchar *)paddedMessage;
  uchar *messageLengthBytes = (uchar *)&messageLength;
  if (isLittleEndian()) {
#pragma unroll
    for (uchar i = 0; i < 60; i += 4) {
      paddedMessageBytes[i] = p[i + 3];
      paddedMessageBytes[i + 1] = p[i + 2];
      paddedMessageBytes[i + 2] = p[i + 1];
      paddedMessageBytes[i + 3] = p[i];
    }
  } else {
#pragma unroll
    for (uchar i = 0; i < 60; ++i) {
      paddedMessageBytes[i] = p[i];
    }
  }

  // 64 bits for the message length
  paddedMessage[15] = messageLength << 3;
}