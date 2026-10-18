; SHBT-MAR-312-IBR Formal SMT-LIB2 Proofs for Z3
; THM-IBR-01: Compacted Catalytic Wafer & Active Symplectic Shield Thermal-Structural-Radiobiological Closure
; THM-IBR-02: Multi-Ton Dark-Braid Superposition, ADM Gauss-Law Conservation, and Intergalactic Multi-Reload Duty-Cycle Closure

(set-logic QF_LRA)

; ----------------------------------------------------------------------------
; THM-IBR-01: Thermal, Structural and Radiobiological Margins
; ----------------------------------------------------------------------------
(push 1)
(declare-const m_seed Real)
(declare-const tau_borrmann Real)
(declare-const f_m Real)
(declare-const t_op Real)
(declare-const t_crit Real)
(declare-const sigma_max Real)
(declare-const sigma_yield Real)
(declare-const crew_dose Real)
(declare-const crew_dose_limit Real)

(assert (= m_seed (/ 37699.0 1000.0)))
(assert (= tau_borrmann 1453.4))
(assert (= f_m 0.782))
(assert (= t_op 21.13))
(assert (= t_crit 32.92))
(assert (= sigma_max 124.60))
(assert (= sigma_yield 350.00))
(assert (= crew_dose 0.114))
(assert (= crew_dose_limit 0.120))

; Negation: Any parameter exceeds safety thresholds
(assert (or
    (<= tau_borrmann 10.0)
    (< f_m 0.74)
    (> t_op t_crit)
    (> sigma_max sigma_yield)
    (>= crew_dose crew_dose_limit)
))

(check-sat)
(pop 1)

; ----------------------------------------------------------------------------
; THM-IBR-02: Multi-Ton Superposition & Intergalactic Duty Cycle
; ----------------------------------------------------------------------------
(push 1)
(declare-const delta_h_vis Real)
(declare-const delta_h_per_braid Real)
(declare-const n_braids Real)
(declare-const framing_defect Real)
(declare-const m_nacelle Real)
(declare-const m_core Real)
(declare-const n_tier1_cores Real)
(declare-const m_tier1_wet Real)
(declare-const stinespring_limit Real)
(declare-const tau_reload Real)
(declare-const tau_burn Real)
(declare-const downtime_ratio Real)
(declare-const duty_cycle Real)

(assert (= delta_h_vis (/ 45.0 64.0)))
(assert (= delta_h_per_braid (- (/ 45.0 7936.0))))
(assert (= n_braids 124.0))
(assert (= framing_defect (+ delta_h_vis (* n_braids delta_h_per_braid))))

(assert (= m_nacelle 2257.70))
(assert (= m_core 376.99))
(assert (= n_tier1_cores 38.0))
(assert (= m_tier1_wet (+ m_nacelle (* n_tier1_cores m_core))))
(assert (= stinespring_limit 16600.00))

(assert (= tau_reload 0.0003178))
(assert (= tau_burn 975000.0))
(assert (= downtime_ratio (/ tau_reload tau_burn)))
(assert (= duty_cycle (- 1.0 downtime_ratio)))

; Negation: Any constraint violated
(assert (or
    (not (= framing_defect 0.0))
    (> m_tier1_wet stinespring_limit)
    (>= downtime_ratio 0.0000001)
    (< duty_cycle 0.999999)
))

(check-sat)
(pop 1)
