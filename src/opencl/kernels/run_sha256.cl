__kernel void run_sha256(uint workersCount, ulong offset,
                         __global uint *foundFlag, __global uchar *output,
                         __global ulong *output_id) {

  const ulong workerId = (ulong)get_global_id(0);

  if (workerId >= workersCount) {
    return;
  }

  const ulong jobId = workerId + offset;

  uchar message[8] = {jobId >> 56, jobId >> 48, jobId >> 40, jobId >> 32,
                      jobId >> 24, jobId >> 16, jobId >> 8,  jobId};

  uchar hashedMessage[32];

  sha256(message, 8, hashedMessage);

  if (hashedMessage[0] != 123 || hashedMessage[1] != 123 ||
      hashedMessage[2] != 123 || hashedMessage[3] != 123) {
    return;
  }

  if (!atomic_cmpxchg(foundFlag, 0, 1)) {
    *output_id = jobId;

#pragma unroll
    for (uchar i = 0; i < 32; ++i) {
      output[i] = hashedMessage[i];
    }
  }
}