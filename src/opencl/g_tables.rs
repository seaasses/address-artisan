use crate::opencl::gpu_cache::{PointGpu, Uint256};
use num_bigint::BigUint;
use num_traits::Zero;
use ocl::{Buffer, Queue};
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use std::sync::OnceLock;

pub const WINDOW_COUNT: usize = 64;
pub const WINDOW_SIZE: usize = 16;
pub const G_TABLES_POINT_COUNT: usize = WINDOW_COUNT * WINDOW_SIZE;

// secp256k1 group order n
const SECP256K1_N_HEX: &[u8] =
    b"FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141";

// Point at infinity sentinel: x = P (an impossible affine x coordinate), y = 0.
// Must match is_infinity_affine_point in the OpenCL code.
const INFINITY_POINT: PointGpu = PointGpu {
    x: Uint256 {
        limbs: [
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFEFFFFFC2F,
        ],
    },
    y: Uint256 { limbs: [0, 0, 0, 0] },
};

static G_TABLES: OnceLock<Vec<PointGpu>> = OnceLock::new();

/// Precomputed tables for fixed-base scalar multiplication on the GPU.
///
/// Flat layout of 64 tables with 16 points each:
/// `tables[w * 16 + d] = d * 16^w * G`, where `w` is the 4-bit window
/// position (0 = least significant nibble of the scalar) and `d` the digit.
/// Entries with `d = 0` hold the point-at-infinity sentinel.
///
/// Computed once per process (~1024 scalar multiplications) and shared by
/// every GPU workbench and test harness.
pub fn g_tables() -> &'static [PointGpu] {
    G_TABLES.get_or_init(compute_g_tables)
}

/// Create a GPU buffer with the precomputed tables. The buffer is written
/// once here and is read-only for the rest of its life; keep it alive for
/// as long as any kernel references it.
pub fn create_g_tables_buffer(queue: &Queue) -> Result<Buffer<PointGpu>, String> {
    let tables = g_tables();
    Buffer::<PointGpu>::builder()
        .queue(queue.clone())
        .len(tables.len())
        .copy_host_slice(tables)
        .build()
        .map_err(|e| format!("Error creating g_times_tables buffer: {}", e))
}

fn compute_g_tables() -> Vec<PointGpu> {
    let secp = Secp256k1::new();
    let n = BigUint::parse_bytes(SECP256K1_N_HEX, 16).expect("valid group order constant");

    let mut tables = Vec::with_capacity(G_TABLES_POINT_COUNT);

    for w in 0..WINDOW_COUNT {
        for d in 0..WINDOW_SIZE {
            if d == 0 {
                tables.push(INFINITY_POINT);
                continue;
            }

            // scalar = d * 16^w mod n (never zero: n is prime and > d, 16)
            let scalar = (BigUint::from(d) << (4 * w)) % &n;
            debug_assert!(!scalar.is_zero());

            let secret = SecretKey::from_byte_array(biguint_to_be_bytes(&scalar))
                .expect("scalar is in [1, n-1]");
            let point = PublicKey::from_secret_key(&secp, &secret);

            let uncompressed = point.serialize_uncompressed();
            tables.push(PointGpu {
                x: uint256_from_be_bytes(&uncompressed[1..33]),
                y: uint256_from_be_bytes(&uncompressed[33..65]),
            });
        }
    }

    tables
}

fn biguint_to_be_bytes(value: &BigUint) -> [u8; 32] {
    let bytes = value.to_bytes_be();
    let mut result = [0u8; 32];
    result[32 - bytes.len()..].copy_from_slice(&bytes);
    result
}

fn uint256_from_be_bytes(bytes: &[u8]) -> Uint256 {
    let mut limbs = [0u64; 4];
    for (limb_index, limb) in limbs.iter_mut().enumerate() {
        let offset = limb_index * 8;
        *limb = u64::from_be_bytes(bytes[offset..offset + 8].try_into().unwrap());
    }
    Uint256 { limbs }
}

#[cfg(test)]
mod tests {
    use super::*;

    const G_X_LIMBS: [u64; 4] = [
        0x79BE667EF9DCBBAC,
        0x55A06295CE870B07,
        0x029BFCDB2DCE28D9,
        0x59F2815B16F81798,
    ];
    const G_Y_LIMBS: [u64; 4] = [
        0x483ADA7726A3C465,
        0x5DA4FBFC0E1108A8,
        0xFD17B448A6855419,
        0x9C47D08FFB10D4B8,
    ];

    #[test]
    fn test_table_size() {
        assert_eq!(g_tables().len(), G_TABLES_POINT_COUNT);
    }

    #[test]
    fn test_zero_digit_entries_are_infinity() {
        let tables = g_tables();
        for w in 0..WINDOW_COUNT {
            assert_eq!(tables[w * WINDOW_SIZE], INFINITY_POINT, "window {}", w);
        }
    }

    #[test]
    fn test_first_table_starts_at_g() {
        let g = &g_tables()[1]; // w = 0, d = 1 -> 1 * G
        assert_eq!(g.x.limbs, G_X_LIMBS);
        assert_eq!(g.y.limbs, G_Y_LIMBS);
    }

    #[test]
    fn test_window_base_is_16_times_previous() {
        // tables[w][1] must equal tables[w-1][16] would need d=16; instead
        // check via scalar identity: tables[1][1] = 16 * G = tables[0][8] + tables[0][8]
        // Simpler exact check: tables[1][1] must equal the point for scalar 16.
        let secp = Secp256k1::new();
        let mut scalar_bytes = [0u8; 32];
        scalar_bytes[31] = 16;
        let expected = PublicKey::from_secret_key(
            &secp,
            &SecretKey::from_byte_array(scalar_bytes).unwrap(),
        )
        .serialize_uncompressed();

        let entry = &g_tables()[WINDOW_SIZE + 1]; // w = 1, d = 1
        assert_eq!(entry.x, uint256_from_be_bytes(&expected[1..33]));
        assert_eq!(entry.y, uint256_from_be_bytes(&expected[33..65]));
    }

    #[test]
    fn test_non_zero_entries_are_valid_points() {
        // Every non-infinity entry must have coordinates < P and not be the sentinel
        let tables = g_tables();
        for (i, point) in tables.iter().enumerate() {
            if i % WINDOW_SIZE == 0 {
                continue;
            }
            assert_ne!(point, &INFINITY_POINT, "entry {} is sentinel", i);
            assert_ne!(point.x.limbs, [0u64; 4], "entry {} has zero x", i);
            assert_ne!(point.y.limbs, [0u64; 4], "entry {} has zero y", i);
        }
    }
}
