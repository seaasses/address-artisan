void ulongToBytes(ulong value, __global uchar *bytes) {
  bytes[0] = value >> 56;
  bytes[1] = (value >> 48);
  bytes[2] = (value >> 40);
  bytes[3] = (value >> 32);
  bytes[4] = (value >> 24);
  bytes[5] = (value >> 16);
  bytes[6] = (value >> 8);
  bytes[7] = value;
}

__kernel void sha512For165BytesMessageKernel(__global uchar *message,
                                       __global uchar *sha512Result) {

  // use the clock as a semi random number
  uchar localMessage[165];
  for (uchar i = 0; i < 165; ++i) {
    localMessage[i] = message[i];
  }

  UInt256 IL, IR;

  sha512For165BytesMessage(localMessage, &IL, &IR);

  ulongToBytes(IL.limbs[0], sha512Result);
  ulongToBytes(IL.limbs[1], sha512Result + 8);
  ulongToBytes(IL.limbs[2], sha512Result + 16);
  ulongToBytes(IL.limbs[3], sha512Result + 24);

  ulongToBytes(IR.limbs[0], sha512Result + 32);
  ulongToBytes(IR.limbs[1], sha512Result + 40);
  ulongToBytes(IR.limbs[2], sha512Result + 48);
  ulongToBytes(IR.limbs[3], sha512Result + 56);
}