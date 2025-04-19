use std::arch::x86_64::*;
use crate::EpiRustError;
use pyo3::prelude::*;

#[derive(Debug)]
pub struct SimdOperations {
    capabilities: SimdCapabilities,
}

#[derive(Debug)]
struct SimdCapabilities {
    sse2_available: bool,
    avx2_available: bool,
    avx512_available: bool,
}

impl SimdCapabilities {
    fn detect() -> Self {
        Self {
            sse2_available: is_x86_feature_detected!("sse2"),
            avx2_available: is_x86_feature_detected!("avx2"),
            avx512_available: is_x86_feature_detected!("avx512f"),
        }
    }
}

impl SimdOperations {
    pub fn new() -> Self {
        Self {
            capabilities: SimdCapabilities::detect(),
        }
    }

    pub fn vector_sum(&self, data: &[f64]) -> Result<f64, EpiRustError> {
        if data.is_empty() {
            return Ok(0.0);
        }

        unsafe {
            if self.capabilities.avx512_available {
                Ok(self.sum_avx512(data))
            } else if self.capabilities.avx2_available {
                Ok(self.sum_avx2(data))
            } else if self.capabilities.sse2_available {
                Ok(self.sum_sse2(data))
            } else {
                Ok(self.sum_scalar(data))
            }
        }
    }

    #[target_feature(enable = "avx512f")]
    unsafe fn sum_avx512(&self, data: &[f64]) -> f64 {
        let mut sum = _mm512_setzero_pd();
        let chunks = data.chunks_exact(8);
        let remainder = chunks.remainder();

        for chunk in chunks {
            let v = _mm512_loadu_pd(chunk.as_ptr());
            sum = _mm512_add_pd(sum, v);
        }

        let mut result = _mm512_reduce_add_pd(sum);
        
        // Handle remaining elements
        for &x in remainder {
            result += x;
        }
        
        result
    }

    #[target_feature(enable = "avx2")]
    unsafe fn sum_avx2(&self, data: &[f64]) -> f64 {
        let mut sum = _mm256_setzero_pd();
        let chunks = data.chunks_exact(4);
        let remainder = chunks.remainder();

        for chunk in chunks {
            let v = _mm256_loadu_pd(chunk.as_ptr());
            sum = _mm256_add_pd(sum, v);
        }

        // Extract and sum the four doubles
        let sum_array = std::mem::transmute::<__m256d, [f64; 4]>(sum);
        let mut result = sum_array.iter().sum::<f64>();

        // Handle remaining elements
        for &x in remainder {
            result += x;
        }
        
        result
    }

    #[target_feature(enable = "sse2")]
    unsafe fn sum_sse2(&self, data: &[f64]) -> f64 {
        let mut sum = _mm_setzero_pd();
        let chunks = data.chunks_exact(2);
        let remainder = chunks.remainder();

        for chunk in chunks {
            let v = _mm_loadu_pd(chunk.as_ptr());
            sum = _mm_add_pd(sum, v);
        }

        // Extract and sum the two doubles
        let sum_array = std::mem::transmute::<__m128d, [f64; 2]>(sum);
        let mut result = sum_array.iter().sum::<f64>();

        // Handle remaining elements
        for &x in remainder {
            result += x;
        }
        
        result
    }

    fn sum_scalar(&self, data: &[f64]) -> f64 {
        data.iter().sum()
    }

    // Optimized survival probability calculation
    pub fn compute_survival_probabilities(
        &self,
        n_risk: &[usize],
        n_event: &[usize]
    ) -> Result<Vec<f64>, EpiRustError> {
        if n_risk.is_empty() || n_event.is_empty() {
            return Ok(vec![1.0]);
        }

        unsafe {
            if self.capabilities.avx2_available {
                self.compute_survival_probabilities_avx2(n_risk, n_event)
            } else if self.capabilities.sse2_available {
                self.compute_survival_probabilities_sse2(n_risk, n_event)
            } else {
                self.compute_survival_probabilities_scalar(n_risk, n_event)
            }
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn compute_survival_probabilities_avx2(
        &self,
        n_risk: &[usize],
        n_event: &[usize]
    ) -> Result<Vec<f64>, EpiRustError> {
        let mut survival = vec![1.0; n_risk.len() + 1];
        let mut current_survival = _mm256_set1_pd(1.0);

        for i in 0..n_risk.len() {
            if n_risk[i] == 0 {
                return Err(EpiRustError::ComputeError(
                    "division by zero in survival probability calculation".into()
                ));
            }

            let prob = (n_risk[i] - n_event[i]) as f64 / n_risk[i] as f64;
            let prob_vec = _mm256_set1_pd(prob);
            current_survival = _mm256_mul_pd(current_survival, prob_vec);
            survival[i + 1] = _mm256_cvtsd_f64(current_survival);
        }

        Ok(survival)
    }

    #[target_feature(enable = "sse2")]
    unsafe fn compute_survival_probabilities_sse2(
        &self,
        n_risk: &[usize],
        n_event: &[usize]
    ) -> Result<Vec<f64>, EpiRustError> {
        let mut survival = vec![1.0; n_risk.len() + 1];
        let mut current_survival = _mm_set1_pd(1.0);

        for i in 0..n_risk.len() {
            if n_risk[i] == 0 {
                return Err(EpiRustError::ComputeError(
                    "division by zero in survival probability calculation".into()
                ));
            }

            let prob = (n_risk[i] - n_event[i]) as f64 / n_risk[i] as f64;
            let prob_vec = _mm_set1_pd(prob);
            current_survival = _mm_mul_pd(current_survival, prob_vec);
            survival[i + 1] = _mm_cvtsd_f64(current_survival);
        }

        Ok(survival)
    }

    fn compute_survival_probabilities_scalar(
        &self,
        n_risk: &[usize],
        n_event: &[usize]
    ) -> Result<Vec<f64>, EpiRustError> {
        let mut survival = vec![1.0; n_risk.len() + 1];
        let mut current_survival = 1.0;

        for i in 0..n_risk.len() {
            if n_risk[i] == 0 {
                return Err(EpiRustError::ComputeError(
                    "division by zero in survival probability calculation".into()
                ));
            }

            let prob = (n_risk[i] - n_event[i]) as f64 / n_risk[i] as f64;
            current_survival *= prob;
            survival[i + 1] = current_survival;
        }

        Ok(survival)
    }
}

pub fn init_submodule(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let submod = PyModule::new(py, "simd")?;
    parent_module.add_submodule(submod)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_vector_sum() {
        let ops = SimdOperations::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sum = ops.vector_sum(&data).unwrap();
        assert_eq!(sum, 15.0);
    }

    #[test]
    fn test_survival_probabilities() {
        let ops = SimdOperations::new();
        let n_risk = vec![100, 90, 80, 70];
        let n_event = vec![10, 5, 8, 7];
        
        let survival = ops.compute_survival_probabilities(&n_risk, &n_event).unwrap();
        
        // Check that probabilities are decreasing
        for i in 1..survival.len() {
            assert!(survival[i] <= survival[i-1]);
        }
        
        // Check bounds
        assert!(survival.iter().all(|&x| x >= 0.0 && x <= 1.0));
    }

    #[test]
    fn test_large_dataset() {
        let ops = SimdOperations::new();
        let mut rng = rand::thread_rng();
        let size = 1000;
        
        let n_risk: Vec<usize> = (1..=size).map(|x| size - x + 1).collect();
        let n_event: Vec<usize> = (0..size).map(|_| rng.gen_range(0..10)).collect();
        
        let survival = ops.compute_survival_probabilities(&n_risk, &n_event).unwrap();
        assert_eq!(survival.len(), size + 1);
    }
} 