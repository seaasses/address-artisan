#define ROTR32(x, n) (((x) >> (n)) | ((x) << (32 - (n))))
#define maj(x, y, z) (((x) & (y)) | ((x) & (z)) | ((y) & (z)))
#define ch(x, y, z) (((x) & (y)) | ((~x) & (z)))
#define sha256SmallSigma0(x) (ROTR32(x, 7) ^ ROTR32(x, 18) ^ (x >> 3))
#define sha256SmallSigma1(x) (ROTR32(x, 17) ^ ROTR32(x, 19) ^ (x >> 10))
#define sha256BigSigma0(x) (ROTR32(x, 2) ^ ROTR32(x, 13) ^ ROTR32(x, 22))
#define sha256BigSigma1(x) (ROTR32(x, 6) ^ ROTR32(x, 11) ^ ROTR32(x, 25))

void sha256(uchar *message, ulong messageLength, uchar *hashedMessage) {

  uint ws[64];

  padMessageSha256(message, messageLength, ws);

#pragma unroll
  for (short i = 16; i < 64; ++i) {
    ws[i] = sha256SmallSigma1(ws[i - 2]) + ws[i - 7] + sha256SmallSigma0(ws[i - 15]) +
            ws[i - 16];
  }

  uint a = 0x6a09e667;
  uint b = 0xbb67ae85;
  uint c = 0x3c6ef372;
  uint d = 0xa54ff53a;
  uint e = 0x510e527f;
  uint f = 0x9b05688c;
  uint g = 0x1f83d9ab;
  uint h = 0x5be0cd19;

  uint t1, t2;
  uint k[64] = {
      0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
      0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
      0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
      0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
      0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
      0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
      0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
      0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
      0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
      0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
      0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2};

#pragma unroll
  for (short t = 0; t < 64; ++t) {
    t1 = h + sha256BigSigma1(e) + ch(e, f, g) + k[t] + ws[t];
    t2 = sha256BigSigma0(a) + maj(a, b, c);

    h = g;
    g = f;
    f = e;
    e = d + t1;
    d = c;
    c = b;
    b = a;
    a = t1 + t2;
  }

  a += 0x6a09e667;
  b += 0xbb67ae85;
  c += 0x3c6ef372;
  d += 0xa54ff53a;
  e += 0x510e527f;
  f += 0x9b05688c;
  g += 0x1f83d9ab;
  h += 0x5be0cd19;

  uchar *aBytes = (uchar *)&a;
  uchar *bBytes = (uchar *)&b;
  uchar *cBytes = (uchar *)&c;
  uchar *dBytes = (uchar *)&d;
  uchar *eBytes = (uchar *)&e;
  uchar *fBytes = (uchar *)&f;
  uchar *gBytes = (uchar *)&g;
  uchar *hBytes = (uchar *)&h;

  if (isLittleEndian()) {
    hashedMessage[0] = aBytes[3];
    hashedMessage[1] = aBytes[2];
    hashedMessage[2] = aBytes[1];
    hashedMessage[3] = aBytes[0];

    hashedMessage[4] = bBytes[3];
    hashedMessage[5] = bBytes[2];
    hashedMessage[6] = bBytes[1];
    hashedMessage[7] = bBytes[0];

    hashedMessage[8] = cBytes[3];
    hashedMessage[9] = cBytes[2];
    hashedMessage[10] = cBytes[1];
    hashedMessage[11] = cBytes[0];

    hashedMessage[12] = dBytes[3];
    hashedMessage[13] = dBytes[2];
    hashedMessage[14] = dBytes[1];
    hashedMessage[15] = dBytes[0];

    hashedMessage[16] = eBytes[3];
    hashedMessage[17] = eBytes[2];
    hashedMessage[18] = eBytes[1];
    hashedMessage[19] = eBytes[0];

    hashedMessage[20] = fBytes[3];
    hashedMessage[21] = fBytes[2];
    hashedMessage[22] = fBytes[1];
    hashedMessage[23] = fBytes[0];

    hashedMessage[24] = gBytes[3];
    hashedMessage[25] = gBytes[2];
    hashedMessage[26] = gBytes[1];
    hashedMessage[27] = gBytes[0];

    hashedMessage[28] = hBytes[3];
    hashedMessage[29] = hBytes[2];
    hashedMessage[30] = hBytes[1];
    hashedMessage[31] = hBytes[0];
  } else {
    hashedMessage[0] = aBytes[0];
    hashedMessage[1] = aBytes[1];
    hashedMessage[2] = aBytes[2];
    hashedMessage[3] = aBytes[3];

    hashedMessage[4] = bBytes[0];
    hashedMessage[5] = bBytes[1];
    hashedMessage[6] = bBytes[2];
    hashedMessage[7] = bBytes[3];

    hashedMessage[8] = cBytes[0];
    hashedMessage[9] = cBytes[1];
    hashedMessage[10] = cBytes[2];
    hashedMessage[11] = cBytes[3];

    hashedMessage[12] = dBytes[0];
    hashedMessage[13] = dBytes[1];
    hashedMessage[14] = dBytes[2];
    hashedMessage[15] = dBytes[3];

    hashedMessage[16] = eBytes[0];
    hashedMessage[17] = eBytes[1];
    hashedMessage[18] = eBytes[2];
    hashedMessage[19] = eBytes[3];

    hashedMessage[20] = fBytes[0];
    hashedMessage[21] = fBytes[1];
    hashedMessage[22] = fBytes[2];
    hashedMessage[23] = fBytes[3];

    hashedMessage[24] = gBytes[0];
    hashedMessage[25] = gBytes[1];
    hashedMessage[26] = gBytes[2];
    hashedMessage[27] = gBytes[3];

    hashedMessage[28] = hBytes[0];
    hashedMessage[29] = hBytes[1];
    hashedMessage[30] = hBytes[2];
    hashedMessage[31] = hBytes[3];
  }
}