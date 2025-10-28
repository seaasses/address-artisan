#include "src/opencl/structs/structs.h"
#include "src/opencl/headers/secp256k1/g_times_scalar.h"
#include "src/opencl/headers/secp256k1/jacobian_to_affine.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"

__kernel void g_times_scalar_compute_kernel(
    __global unsigned char *scalar_buffer,
    __global int *max_threads,
    __global int *output,
    __global ulong *iteration_offset)
{
    int thread_id = get_global_id(0);
    ulong offset = *iteration_offset;

    // Copy data from global to private memory and convert
    unsigned char scalar_private[32];
    int i;
    for (i = 0; i < 32; i++) {
        scalar_private[i] = scalar_buffer[i];
    }

    Uint256 scalar;

    // Convert byte arrays to Uint256
    bytes_to_uint256(scalar_private, &scalar);

    // Add thread ID AND iteration offset to make EVERY execution different
    // Isso evita que o compilador/GPU cache os resultados entre kernels
    scalar.limbs[0] += offset;              // Offset muda a cada kernel
    scalar.limbs[1] += (ulong)thread_id;    // Thread ID muda dentro do kernel
    scalar.limbs[2] += (ulong)thread_id + offset;
    scalar.limbs[3] += (ulong)thread_id;

    // Perform g times scalar multiplication (returns Jacobian point)
    JacobianPoint jacobian_result = g_times_scalar(scalar);

    // Convert to affine coordinates
    Point result = jacobian_to_affine(jacobian_result);

    // FORÇA a GPU a escrever resultados únicos para cada thread
    // Cada thread escreve em uma posição diferente do array de output
    // O compilador NÃO pode otimizar isso!

    // Usa apenas o bit menos significativo de cada coordenada
    // Escreve na posição do thread_id (mod tamanho do buffer)
    int value = (int)((result.x.limbs[0] ^ result.y.limbs[0]) & 0xFF);

    // Cada thread escreve na sua posição
    output[thread_id % (*max_threads)] = value;
}