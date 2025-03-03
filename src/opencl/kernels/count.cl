__kernel void count(uint workers_count, ulong offset, __global uint *found_flag,
                    __global uchar *output) {

  const ulong worker_id = (ulong)get_global_id(0);

  if (worker_id >= workers_count ) {
    return;
  }

  const ulong job_id = worker_id + offset;

  uchar message[8] = {job_id >> 56, job_id >> 48, job_id >> 40, job_id >> 32,
                      job_id >> 24, job_id >> 16, job_id >> 8,  job_id};

  uchar hashedMessage[32];

  sha256(message, 8, hashedMessage);

  if (hashedMessage[0] != 0x00 || hashedMessage[1] != 0x00 ||
      hashedMessage[2] != 190 || hashedMessage[3] != 185 ||
      hashedMessage[4] != 173) {
    return;
  }

  if (!atomic_cmpxchg(found_flag, 0, 1)) {

#pragma unroll
    for (uchar i = 0; i < 32; ++i) {
      output[i] = hashedMessage[i];
    }
  }
}