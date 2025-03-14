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
  b += 0xbb67ae8584caa73b;
  c += 0x3c6ef372fe94f82b;
  d += 0xa54ff53a5f1d36f1;
  e += 0x510e527fade682d1;
  f += 0x9b05688c2b3e6c1f;
  g += 0x1f83d9abfb41bd6b;
  h += 0x5be0cd19137e2179;

  hashedMessage[0] = a >> 56;
  hashedMessage[1] = (a >> 48);
  hashedMessage[2] = (a >> 40);
  hashedMessage[3] = (a >> 32);
  hashedMessage[4] = (a >> 24);
  hashedMessage[5] = (a >> 16);
  hashedMessage[6] = (a >> 8);
  hashedMessage[7] = a;

  hashedMessage[8] = b >> 56;
  hashedMessage[9] = b >> 48;
  hashedMessage[10] = b >> 40;
  hashedMessage[11] = b >> 32;
  hashedMessage[12] = b >> 24;
  hashedMessage[13] = b >> 16;
  hashedMessage[14] = b >> 8;
  hashedMessage[15] = b;

  hashedMessage[16] = c >> 56;
  hashedMessage[17] = c >> 48;
  hashedMessage[18] = c >> 40;
  hashedMessage[19] = c >> 32;
  hashedMessage[20] = c >> 24;
  hashedMessage[21] = c >> 16;
  hashedMessage[22] = c >> 8;
  hashedMessage[23] = c;

  hashedMessage[24] = d >> 56;
  hashedMessage[25] = d >> 48;
  hashedMessage[26] = d >> 40;
  hashedMessage[27] = d >> 32;
  hashedMessage[28] = d >> 24;
  hashedMessage[29] = d >> 16;
  hashedMessage[30] = d >> 8;
  hashedMessage[31] = d;

  hashedMessage[32] = e >> 56;
  hashedMessage[33] = e >> 48;
  hashedMessage[34] = e >> 40;
  hashedMessage[35] = e >> 32;
  hashedMessage[36] = e >> 24;
  hashedMessage[37] = e >> 16;
  hashedMessage[38] = e >> 8;
  hashedMessage[39] = e;

  hashedMessage[40] = f >> 56;
  hashedMessage[41] = f >> 48;
  hashedMessage[42] = f >> 40;
  hashedMessage[43] = f >> 32;
  hashedMessage[44] = f >> 24;
  hashedMessage[45] = f >> 16;
  hashedMessage[46] = f >> 8;
  hashedMessage[47] = f;

  hashedMessage[48] = g >> 56;
  hashedMessage[49] = g >> 48;
  hashedMessage[50] = g >> 40;
  hashedMessage[51] = g >> 32;
  hashedMessage[52] = g >> 24;
  hashedMessage[53] = g >> 16;
  hashedMessage[54] = g >> 8;
  hashedMessage[55] = g;

  hashedMessage[56] = h >> 56;
  hashedMessage[57] = h >> 48;
  hashedMessage[58] = h >> 40;
  hashedMessage[59] = h >> 32;
  hashedMessage[60] = h >> 24;
  hashedMessage[61] = h >> 16;
  hashedMessage[62] = h >> 8;
  hashedMessage[63] = h;
}