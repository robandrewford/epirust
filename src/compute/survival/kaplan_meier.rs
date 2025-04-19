use crate::compute::simd::SimdOperations;
use crate::compute::parallel::NumaAwareThreadPool;
use ndarray::{Array1, Array2};
use std::arch::x86_64::*;

#[derive(Debug)]
pub struct KaplanMeier {
    simd_ops: SimdOperations,
    thread_pool: NumaAwareThreadPool,
}

#[derive(Debug)]
pub struct KMResult {
    pub time: Vec<f64>,
    pub survival: Vec<f64>,
    pub std_error: Vec<f64>,
    pub n_risk: Vec<usize>,
    pub n_event: Vec<usize>,
}

impl KaplanMeier {
    pub fn new() -> Result<Self, crate::EpiRustError> {
        Ok(Self {
            simd_ops: SimdOperations::new(),
            thread_pool: NumaAwareThreadPool::new(Default::default())?,
        })
    }

    pub fn fit(&self, time: &[f64], event: &[bool]) -> Result<KMResult, crate::EpiRustError> {
        if time.len() != event.len() {
            return Err(crate::EpiRustError::ComputeError(
                "time and event vectors must have same length".into()
            ));
        }

        // Sort times and get unique times with events
        let mut sorted_data: Vec<(f64, bool)> = time.iter()
            .zip(event.iter())
            .map(|(&t, &e)| (t, e))
            .collect();
        sorted_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Use SIMD for counting at-risk and events
        let (unique_times, n_risk, n_event) = self.thread_pool.run(&sorted_data, |data| {
            self.compute_risk_and_events(data)
        })?;

        // Compute survival probabilities using SIMD
        let survival = self.compute_survival_probabilities(&n_risk, &n_event)?;
        
        // Compute standard errors in parallel
        let std_error = self.compute_standard_errors(&survival, &n_risk, &n_event)?;

        Ok(KMResult {
            time: unique_times,
            survival,
            std_error,
            n_risk,
            n_event,
        })
    }

    #[target_feature(enable = "avx2")]
    unsafe fn compute_survival_probabilities(
        &self,
        n_risk: &[usize],
        n_event: &[usize]
    ) -> Result<Vec<f64>, crate::EpiRustError> {
        let mut survival = vec![1.0];
        let mut current_survival = _mm256_set1_pd(1.0);

        for ((&at_risk, &events), surv) in n_risk.iter()
            .zip(n_event.iter())
            .zip(survival.iter_mut().skip(1))
        {
            if at_risk == 0 {
                return Err(crate::EpiRustError::ComputeError(
                    "division by zero in survival probability calculation".into()
                ));
            }

            let prob = _mm256_set1_pd((at_risk - events) as f64 / at_risk as f64);
            current_survival = _mm256_mul_pd(current_survival, prob);
            *surv = _mm256_cvtsd_f64(current_survival);
        }

        Ok(survival)
    }

    fn compute_risk_and_events(
        &self,
        data: &[(f64, bool)]
    ) -> Result<(Vec<f64>, Vec<usize>, Vec<usize>), crate::EpiRustError> {
        let mut unique_times = Vec::new();
        let mut n_risk = Vec::new();
        let mut n_event = Vec::new();

        let mut current_time = data[0].0;
        let mut current_risk = data.len();
        let mut current_events = 0;

        for &(time, event) in data {
            if time != current_time {
                unique_times.push(current_time);
                n_risk.push(current_risk);
                n_event.push(current_events);
                
                current_time = time;
                current_events = 0;
            }
            
            if event {
                current_events += 1;
            }
            current_risk -= 1;
        }

        // Add last group
        unique_times.push(current_time);
        n_risk.push(current_risk);
        n_event.push(current_events);

        Ok((unique_times, n_risk, n_event))
    }

    fn compute_standard_errors(
        &self,
        survival: &[f64],
        n_risk: &[usize],
        n_event: &[usize]
    ) -> Result<Vec<f64>, crate::EpiRustError> {
        // Greenwood's formula
        let mut std_error = vec![0.0; survival.len()];
        
        self.thread_pool.run(&survival, |_| {
            for i in 0..survival.len() {
                if n_risk[i] == 0 {
                    continue;
                }
                
                let variance = n_event[i] as f64 / 
                    ((n_risk[i] * (n_risk[i] - n_event[i])) as f64);
                std_error[i] = (survival[i] * variance.sqrt()).abs();
            }
        });

        Ok(std_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datasets::test_data;

    #[test]
    fn test_kaplan_meier_basic() {
        let km = KaplanMeier::new().unwrap();
        let time = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let event = vec![false, true, false, true, false];
        
        let result = km.fit(&time, &event).unwrap();
        
        assert_eq!(result.time.len(), result.survival.len());
        assert!(result.survival.iter().all(|&s| s >= 0.0 && s <= 1.0));
    }

    #[test]
    fn test_kaplan_meier_empty() {
        let km = KaplanMeier::new().unwrap();
        let result = km.fit(&[], &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kaplan_meier_all_censored() {
        let km = KaplanMeier::new().unwrap();
        let time = vec![1.0, 2.0, 3.0];
        let event = vec![false, false, false];
        
        let result = km.fit(&time, &event).unwrap();
        assert!(result.survival.iter().all(|&s| (s - 1.0).abs() < 1e-10));
    }
} 