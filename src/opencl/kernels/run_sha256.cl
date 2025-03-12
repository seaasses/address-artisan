__kernel void sha256_kernel(__global uchar* message, uint messageLength, __global uchar* sha256Result) {

  const ulong workerId = (ulong)get_global_id(0);
  uchar localMessage[55];

  for (uint i = 0; i < messageLength; i++) {
    localMessage[i] = message[i];
  }

  if (workerId > 0) {
    return;
  }

  uchar hashedMessage[32];

  sha256(localMessage, (ulong)messageLength, hashedMessage);

#pragma unroll
  for (uchar i = 0; i < 32; ++i) {
    sha256Result[i] = hashedMessage[i];
  }
}