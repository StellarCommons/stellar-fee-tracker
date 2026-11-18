//! Small shared RNG and statistics helpers for the simulation modules.

/// Arithmetic mean; `0.0` for an empty slice.
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// One step of xorshift64* returning a value in `[0, 1)`.
pub fn next_unit(rng: &mut u64) -> f64 {
    let mut x = *rng;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    *rng = x;
    let value = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
    (value >> 11) as f64 / (1u64 << 53) as f64
}
