#include "src/opencl/headers/sha256/padMessageSha256.h"

#define ROTR32(x, n) (((x) >> (n)) | ((x) << (32 - (n))))
#define maj(x, y, z) (((x) & (y)) | ((x) & (z)) | ((y) & (z)))
#define ch(x, y, z) (((x) & (y)) | ((~x) & (z)))
#define sha256SmallSigma0(x) (ROTR32(x, 7) ^ ROTR32(x, 18) ^ (x >> 3))
#define sha256SmallSigma1(x) (ROTR32(x, 17) ^ ROTR32(x, 19) ^ (x >> 10))
#define sha256BigSigma0(x) (ROTR32(x, 2) ^ ROTR32(x, 13) ^ ROTR32(x, 22))
#define sha256BigSigma1(x) (ROTR32(x, 6) ^ ROTR32(x, 11) ^ ROTR32(x, 25))

void sha256(unsigned char *message, unsigned long messageLength, unsigned char *hashedMessage)
{

  unsigned int ws[64];

  padMessageSha256(message, messageLength, ws);

#pragma unroll
  for (short i = 16; i < 64; ++i)
  {
    ws[i] = sha256SmallSigma1(ws[i - 2]) + ws[i - 7] +
            sha256SmallSigma0(ws[i - 15]) + ws[i - 16];
  }

  unsigned int a = 0x6a09e667;
  unsigned int b = 0xbb67ae85;
  unsigned int c = 0x3c6ef372;
  unsigned int d = 0xa54ff53a;
  unsigned int e = 0x510e527f;
  unsigned int f = 0x9b05688c;
  unsigned int g = 0x1f83d9ab;
  unsigned int h = 0x5be0cd19;

  unsigned int t1, t2;
  const unsigned int k[64] = {
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
  for (short t = 0; t < 64; ++t)
  {
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

  hashedMessage[0] = a >> 24;
  hashedMessage[1] = (a >> 16);
  hashedMessage[2] = (a >> 8);
  hashedMessage[3] = a;

  hashedMessage[4] = b >> 24;
  hashedMessage[5] = (b >> 16);
  hashedMessage[6] = (b >> 8);
  hashedMessage[7] = b;

  hashedMessage[8] = c >> 24;
  hashedMessage[9] = (c >> 16);
  hashedMessage[10] = (c >> 8);
  hashedMessage[11] = c;

  hashedMessage[12] = d >> 24;
  hashedMessage[13] = (d >> 16);
  hashedMessage[14] = (d >> 8);
  hashedMessage[15] = d;

  hashedMessage[16] = e >> 24;
  hashedMessage[17] = (e >> 16);
  hashedMessage[18] = (e >> 8);
  hashedMessage[19] = e;

  hashedMessage[20] = f >> 24;
  hashedMessage[21] = (f >> 16);
  hashedMessage[22] = (f >> 8);
  hashedMessage[23] = f;

  hashedMessage[24] = g >> 24;
  hashedMessage[25] = (g >> 16);
  hashedMessage[26] = (g >> 8);
  hashedMessage[27] = g;

  hashedMessage[28] = h >> 24;
  hashedMessage[29] = (h >> 16);
  hashedMessage[30] = (h >> 8);
  hashedMessage[31] = h;
}