__kernel void sha512_kernel(__global uchar *message, uint messageLength,
                            __global uchar *sha512Result) {

  const ulong workerId = (ulong)get_global_id(0);
  uchar localMessage[111];

  for (uint i = 0; i < messageLength; i++) {
    localMessage[i] = message[i];
  }

  if (workerId > 0) {
    return;
  }

  uchar hashedMessage[64];

  sha512(localMessage, (ulong)messageLength, hashedMessage);

#pragma unroll
  for (uchar i = 0; i < 64; ++i) {
    sha512Result[i] = hashedMessage[i];
  }
}