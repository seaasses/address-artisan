#define ROTR64(x, n) (((x) >> (n)) | ((x) << (64 - (n))))
#define maj(x, y, z) (((x) & (y)) | ((x) & (z)) | ((y) & (z)))
#define ch(x, y, z) (((x) & (y)) | ((~x) & (z)))
#define sha512SmallSigma0(x) (ROTR64(x, 1) ^ ROTR64(x, 8) ^ (x >> 7))
#define sha512SmallSigma1(x) (ROTR64(x, 19) ^ ROTR64(x, 61) ^ (x >> 6))
#define sha512BigSigma0(x) (ROTR64(x, 28) ^ ROTR64(x, 34) ^ ROTR64(x, 39))
#define sha512BigSigma1(x) (ROTR64(x, 14) ^ ROTR64(x, 18) ^ ROTR64(x, 41))

void sha512(uchar *message, ulong messageLength, uchar *hashedMessage) {

  ulong ws[80]; // 8 * 80 = 640 bytes =

  padMessageSha512(message, messageLength, ws);

#pragma unroll
  for (short i = 16; i < 80; ++i) {
    ws[i] = sha512SmallSigma1(ws[i - 2]) + ws[i - 7] +
            sha512SmallSigma0(ws[i - 15]) + ws[i - 16];
  }
  ulong a = 0x6a09e667f3bcc908;
  ulong b = 0xbb67ae8584caa73b;
  ulong c = 0x3c6ef372fe94f82b;
  ulong d = 0xa54ff53a5f1d36f1;
  ulong e = 0x510e527fade682d1;
  ulong f = 0x9b05688c2b3e6c1f;
  ulong g = 0x1f83d9abfb41bd6b;
  ulong h = 0x5be0cd19137e2179;

  ulong t1, t2;

  ulong k[80] = {0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f,
                 0xe9b5dba58189dbbc, 0x3956c25bf348b538, 0x59f111f1b605d019,
                 0x923f82a4af194f9b, 0xab1c5ed5da6d8118, 0xd807aa98a3030242,
                 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
                 0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235,
                 0xc19bf174cf692694, 0xe49b69c19ef14ad2, 0xefbe4786384f25e3,
                 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65, 0x2de92c6f592b0275,
                 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
                 0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f,
                 0xbf597fc7beef0ee4, 0xc6e00bf33da88fc2, 0xd5a79147930aa725,
                 0x06ca6351e003826f, 0x142929670a0e6e70, 0x27b70a8546d22ffc,
                 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
                 0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6,
                 0x92722c851482353b, 0xa2bfe8a14cf10364, 0xa81a664bbc423001,
                 0xc24b8b70d0f89791, 0xc76c51a30654be30, 0xd192e819d6ef5218,
                 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
                 0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99,
                 0x34b0bcb5e19b48a8, 0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb,
                 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3, 0x748f82ee5defb2fc,
                 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
                 0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915,
                 0xc67178f2e372532b, 0xca273eceea26619c, 0xd186b8c721c0c207,
                 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178, 0x06f067aa72176fba,
                 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
                 0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc,
                 0x431d67c49c100d4c, 0x4cc5d4becb3e42b6, 0x597f299cfc657e2a,
                 0x5fcb6fab3ad6faec, 0x6c44198c4a475817};

#pragma unroll
  for (short t = 0; t < 80; ++t) {
    t1 = h + sha512BigSigma1(e) + ch(e, f, g) + k[t] + ws[t];
    t2 = sha512BigSigma0(a) + maj(a, b, c);
    h = g;
    g = f;
    f = e;
    e = d + t1;
    d = c;
    c = b;
    b = a;
    a = t1 + t2;
  }


  a += 0x6a09e667f3bcc908;
  b += 0xbb67ae858dd64aeb;
  c += 0x3c6ef372fe94f82b;
  d += 0xa54ff53a5f1d36f1;
  e += 0x510e527fade682d1;
  f += 0x9b05688c2b3e6c1f;
  g += 0x1f83d9abfb41bd6b;
  h += 0x5be0cd19137e2179;

  uchar *aBytes = (uchar *)&a;
  uchar *bBytes = (uchar *)&b;
  uchar *cBytes = (uchar *)&c;
  uchar *dBytes = (uchar *)&d;
  uchar *eBytes = (uchar *)&e;
  uchar *fBytes = (uchar *)&f;
  uchar *gBytes = (uchar *)&g;
  uchar *hBytes = (uchar *)&h;

  if (isLittleEndian()) {
    hashedMessage[0] = aBytes[7];
    hashedMessage[1] = aBytes[6];
    hashedMessage[2] = aBytes[5];
    hashedMessage[3] = aBytes[4];
    hashedMessage[4] = aBytes[3];
    hashedMessage[5] = aBytes[2];
    hashedMessage[6] = aBytes[1];
    hashedMessage[7] = aBytes[0];

    hashedMessage[8] = bBytes[7];
    hashedMessage[9] = bBytes[6];
    hashedMessage[10] = bBytes[5];
    hashedMessage[11] = bBytes[4];
    hashedMessage[12] = bBytes[3];
    hashedMessage[13] = bBytes[2];
    hashedMessage[14] = bBytes[1];
    hashedMessage[15] = bBytes[0];

    hashedMessage[16] = cBytes[7];
    hashedMessage[17] = cBytes[6];
    hashedMessage[18] = cBytes[5];
    hashedMessage[19] = cBytes[4];
    hashedMessage[20] = cBytes[3];
    hashedMessage[21] = cBytes[2];
    hashedMessage[22] = cBytes[1];
    hashedMessage[23] = cBytes[0];

    hashedMessage[24] = dBytes[7];
    hashedMessage[25] = dBytes[6];
    hashedMessage[26] = dBytes[5];
    hashedMessage[27] = dBytes[4];
    hashedMessage[28] = dBytes[3];
    hashedMessage[29] = dBytes[2];
    hashedMessage[30] = dBytes[1];
    hashedMessage[31] = dBytes[0];

    hashedMessage[32] = eBytes[7];
    hashedMessage[33] = eBytes[6];
    hashedMessage[34] = eBytes[5];
    hashedMessage[35] = eBytes[4];
    hashedMessage[36] = eBytes[3];
    hashedMessage[37] = eBytes[2];
    hashedMessage[38] = eBytes[1];
    hashedMessage[39] = eBytes[0];

    hashedMessage[40] = fBytes[7];
    hashedMessage[41] = fBytes[6];
    hashedMessage[42] = fBytes[5];
    hashedMessage[43] = fBytes[4];
    hashedMessage[44] = fBytes[3];
    hashedMessage[45] = fBytes[2];
    hashedMessage[46] = fBytes[1];
    hashedMessage[47] = fBytes[0];

    hashedMessage[48] = gBytes[7];
    hashedMessage[49] = gBytes[6];
    hashedMessage[50] = gBytes[5];
    hashedMessage[51] = gBytes[4];
    hashedMessage[52] = gBytes[3];
    hashedMessage[53] = gBytes[2];
    hashedMessage[54] = gBytes[1];
    hashedMessage[55] = gBytes[0];

    hashedMessage[56] = hBytes[7];
    hashedMessage[57] = hBytes[6];
    hashedMessage[58] = hBytes[5];
    hashedMessage[59] = hBytes[4];
    hashedMessage[60] = hBytes[3];
    hashedMessage[61] = hBytes[2];
    hashedMessage[62] = hBytes[1];
    hashedMessage[63] = hBytes[0];

  } else {
    hashedMessage[0] = aBytes[0];
    hashedMessage[1] = aBytes[1];
    hashedMessage[2] = aBytes[2];
    hashedMessage[3] = aBytes[3];
    hashedMessage[4] = aBytes[4];
    hashedMessage[5] = aBytes[5];
    hashedMessage[6] = aBytes[6];
    hashedMessage[7] = aBytes[7];

    hashedMessage[8] = bBytes[0];
    hashedMessage[9] = bBytes[1];
    hashedMessage[10] = bBytes[2];
    hashedMessage[11] = bBytes[3];
    hashedMessage[12] = bBytes[4];
    hashedMessage[13] = bBytes[5];
    hashedMessage[14] = bBytes[6];
    hashedMessage[15] = bBytes[7];

    hashedMessage[16] = cBytes[0];
    hashedMessage[17] = cBytes[1];
    hashedMessage[18] = cBytes[2];
    hashedMessage[19] = cBytes[3];
    hashedMessage[20] = cBytes[4];
    hashedMessage[21] = cBytes[5];
    hashedMessage[22] = cBytes[6];
    hashedMessage[23] = cBytes[7];

    hashedMessage[24] = dBytes[0];
    hashedMessage[25] = dBytes[1];
    hashedMessage[26] = dBytes[2];
    hashedMessage[27] = dBytes[3];
    hashedMessage[28] = dBytes[4];
    hashedMessage[29] = dBytes[5];
    hashedMessage[30] = dBytes[6];
    hashedMessage[31] = dBytes[7];

    hashedMessage[32] = eBytes[0];
    hashedMessage[33] = eBytes[1];
    hashedMessage[34] = eBytes[2];
    hashedMessage[35] = eBytes[3];
    hashedMessage[36] = eBytes[4];
    hashedMessage[37] = eBytes[5];
    hashedMessage[38] = eBytes[6];
    hashedMessage[39] = eBytes[7];

    hashedMessage[40] = fBytes[0];
    hashedMessage[41] = fBytes[1];
    hashedMessage[42] = fBytes[2];
    hashedMessage[43] = fBytes[3];
    hashedMessage[44] = fBytes[4];
    hashedMessage[45] = fBytes[5];
    hashedMessage[46] = fBytes[6];
    hashedMessage[47] = fBytes[7];

    hashedMessage[48] = gBytes[0];
    hashedMessage[49] = gBytes[1];
    hashedMessage[50] = gBytes[2];
    hashedMessage[51] = gBytes[3];
    hashedMessage[52] = gBytes[4];
    hashedMessage[53] = gBytes[5];
    hashedMessage[54] = gBytes[6];
    hashedMessage[55] = gBytes[7];

    hashedMessage[56] = hBytes[0];
    hashedMessage[57] = hBytes[1];
    hashedMessage[58] = hBytes[2];
    hashedMessage[59] = hBytes[3];
    hashedMessage[60] = hBytes[4];
    hashedMessage[61] = hBytes[5];
    hashedMessage[62] = hBytes[6];
    hashedMessage[63] = hBytes[7];
  }
}