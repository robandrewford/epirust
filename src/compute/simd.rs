use std::simd::{f32x8, Simd};

pub struct SimdOperations;

impl SimdOperations {
    pub fn compute_survival_probabilities(&self, n_risk: &[f32], n_event: &[f32]) -> Vec<f32> {
        let len = n_risk.len();
        let mut survival = vec![1.0; len];
        
        // Process 8 elements at a time using SIMD
        let simd_len = len - (len % 8);
        for i in (0..simd_len).step_by(8) {
            let n_risk_simd = f32x8::from_slice(&n_risk[i..i + 8]);
            let n_event_simd = f32x8::from_slice(&n_event[i..i + 8]);
            
            // Calculate (n_risk - n_event) / n_risk for 8 elements at once
            let prob_simd = (n_risk_simd - n_event_simd) / n_risk_simd;
            
            // Store results back to survival vector
            prob_simd.store_unaligned(&mut survival[i..i + 8]);
            
            // Update cumulative survival probabilities
            if i > 0 {
                let prev_survival = survival[i - 1];
                for j in i..i + 8 {
                    survival[j] *= prev_survival;
                }
            }
        }
        
        // Handle remaining elements
        for i in simd_len..len {
            let prob = (n_risk[i] - n_event[i]) / n_risk[i];
            survival[i] = if i > 0 { survival[i - 1] * prob } else { prob };
        }
        
        survival
    }
} 