//! SHBT Holographic Warp Drive Simulator — Rust/PyO3 core.

pub mod boundary;
pub mod causal_observer;
pub mod constants;
pub mod derender;
pub mod projector;
pub mod stress_energy;
pub mod thermodynamics;

pub use boundary::*;
pub use causal_observer::*;
pub use constants::*;
pub use derender::*;
pub use projector::*;
pub use stress_energy::*;
pub use thermodynamics::*;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

fn audit_to_pydict<'py>(py: Python<'py>, audit: &HashMap<String, f64>) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    for (k, v) in audit {
        dict.set_item(k.as_str(), *v)?;
    }
    Ok(dict)
}

fn vec_to_pylist<'py>(py: Python<'py>, v: &[f64]) -> PyResult<Bound<'py, pyo3::types::PyList>> {
    let list = pyo3::types::PyList::new(py, v.iter().copied().collect::<Vec<_>>())?;
    Ok(list)
}

fn vec_dict_to_pydict<'py>(py: Python<'py>, data: &HashMap<String, Vec<f64>>) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    for (k, v) in data {
        dict.set_item(k.as_str(), vec_to_pylist(py, v)?)?;
    }
    Ok(dict)
}

/// High-level simulation container exposed to Python.
#[pyclass(name = "Simulation")]
pub struct Simulation {
    #[pyo3(get)]
    pub radius: f64,
    #[pyo3(get)]
    pub domain_radius: f64,
    #[pyo3(get)]
    pub grid_points: usize,
    #[pyo3(get)]
    pub stress_grid_points: usize,
    #[pyo3(get)]
    pub wall_steepness: f64,
    #[pyo3(get)]
    pub phase: f64,
}

#[pymethods]
impl Simulation {
    #[new]
    #[pyo3(signature = (radius=10.0, domain_radius=30.0, grid_points=1201, wall_steepness=0.8, phase=0.421, stress_grid_points=7))]
    fn new(
        radius: f64,
        domain_radius: f64,
        grid_points: usize,
        wall_steepness: f64,
        phase: f64,
        stress_grid_points: usize,
    ) -> Self {
        Simulation {
            radius,
            domain_radius,
            grid_points,
            stress_grid_points,
            wall_steepness,
            phase,
        }
    }

    /// Run all audits and return a Python dict of results.
    #[pyo3(signature = (tolerance=1.0e-12))]
    fn run<'py>(&self, py: Python<'py>, tolerance: f64) -> PyResult<Bound<'py, PyDict>> {
        let out = PyDict::new(py);

        // Boundary and excitation.
        let boundary = BoundaryRegister::new();
        let engine = ExcitationEngine::new(boundary.clone(), self.phase);

        let boundary_dict = PyDict::new(py);
        let (l, q, p) = boundary.branch();
        boundary_dict.set_item("lepton_level", l)?;
        boundary_dict.set_item("quark_level", q)?;
        boundary_dict.set_item("parent_level", p)?;
        boundary_dict.set_item("c_dark_residual", C_DARK_RESIDUAL)?;
        boundary_dict.set_item("framing_defect", engine.framing_defect())?;
        boundary_dict.set_item("closure_norm", engine.closure_norm())?;
        boundary_dict.set_item(
            "shannon_density",
            boundary
                .shannon_density
                .iter()
                .flat_map(|row| row.iter().copied())
                .collect::<Vec<f64>>(),
        )?;
        boundary_dict.set_item("shannon_entropy", boundary.shannon_entropy)?;
        boundary_dict.set_item("audit", audit_to_pydict(py, &boundary.audit(tolerance))?)?;
        out.set_item("boundary", boundary_dict)?;

        let population_shift_l1 = {
            let probs = engine.excited_probability();
            let mut sum = 0.0;
            for i in 0..3 {
                for j in 0..3 {
                    sum += (probs[flatten_index(i, j)] - boundary.rho_e[i][j]).abs();
                }
            }
            sum
        };

        let excitation_dict = PyDict::new(py);
        excitation_dict.set_item("population_shift_l1", population_shift_l1)?;
        excitation_dict.set_item("framing_defect", engine.framing_defect())?;
        excitation_dict.set_item("closure_norm", engine.closure_norm())?;
        excitation_dict.set_item(
            "excited_shannon_contributions",
            engine
                .excited_shannon_contributions()
                .iter()
                .flat_map(|row| row.iter().copied())
                .collect::<Vec<f64>>(),
        )?;
        excitation_dict.set_item("audit", audit_to_pydict(py, &engine.audit(tolerance))?)?;
        out.set_item("excitation", excitation_dict)?;

        // FG slice projector.
        let fg = FGSliceProjector::new(
            DELTA_MOD,
            self.radius,
            self.domain_radius,
            self.grid_points,
            self.wall_steepness,
        );
        let fg_dict = PyDict::new(py);
        fg_dict.set_item("v_eff_c", fg.v_eff_c())?;
        fg_dict.set_item("speed_of_light_m_s", LIGHT_SPEED_M_S)?;
        fg_dict.set_item("x_m", vec_to_pylist(py, fg.x_m.as_slice().unwrap_or(&[]))?)?;
        fg_dict.set_item("shape", vec_to_pylist(py, fg.shape.as_slice().unwrap_or(&[]))?)?;
        fg_dict.set_item(
            "beta_m_s",
            vec_to_pylist(py, fg.beta_m_s.as_slice().unwrap_or(&[]))?,
        )?;
        let beta_over_c: Vec<f64> = fg.beta_m_s.iter().map(|b| b / fg.speed_of_light_m_s).collect();
        fg_dict.set_item("beta_over_c", vec_to_pylist(py, &beta_over_c)?)?;
        fg_dict.set_item("audit", audit_to_pydict(py, &fg.audit(tolerance))?)?;
        out.set_item("fg_slice", fg_dict)?;

        // 3-D metric calculator.
        let metric3d = Metric3DCalculator::new(
            self.radius,
            self.domain_radius,
            self.stress_grid_points,
            self.wall_steepness,
            DELTA_MOD,
        );
        let n3 = metric3d.grid_points_per_axis;
        let mid3 = n3 / 2;
        let x3: Vec<f64> = metric3d.x_m.iter().copied().collect();
        let shape_line: Vec<f64> = (0..n3).map(|i| metric3d.shape[[i, mid3, mid3]]).collect();
        let beta_line: Vec<f64> =
            (0..n3).map(|i| metric3d.beta_over_c[[i, mid3, mid3]]).collect();

        let metric3d_dict = PyDict::new(py);
        metric3d_dict.set_item("grid_points_per_axis", n3)?;
        metric3d_dict.set_item("x_m", vec_to_pylist(py, &x3)?)?;
        metric3d_dict.set_item("shape_center_line", vec_to_pylist(py, &shape_line)?)?;
        metric3d_dict.set_item("beta_center_line", vec_to_pylist(py, &beta_line)?)?;
        let metric3d_audit = metric3d.audit(tolerance);
        metric3d_dict.set_item("audit", audit_to_pydict(py, &metric3d_audit)?)?;
        out.set_item("metric3d", metric3d_dict)?;

        // Stress-energy auditor.
        let auditor = StressEnergyAuditor::new(
            metric3d.metric_4d_grid.clone(),
            (metric3d.x_m.clone(), metric3d.y_m.clone(), metric3d.z_m.clone()),
            LAMBDA_HOLO_SI,
            GRAVITATIONAL_CONSTANT_SI,
        );
        let stress_dict = PyDict::new(py);
        let ricci_line: Vec<f64> =
            (0..n3).map(|i| auditor.ricci_scalar[[i, mid3, mid3]]).collect();
        let t00_line: Vec<f64> =
            (0..n3).map(|i| auditor.stress_energy[[i, mid3, mid3, 0, 0]]).collect();
        stress_dict.set_item("x_m", vec_to_pylist(py, &x3)?)?;
        stress_dict.set_item("ricci_scalar", vec_to_pylist(py, &ricci_line)?)?;
        stress_dict.set_item("energy_density_t00", vec_to_pylist(py, &t00_line)?)?;
        stress_dict.set_item(
            "audit",
            audit_to_pydict(py, &auditor.audit_energy_conditions(100, 26008312, tolerance))?,
        )?;
        out.set_item("stress_energy", stress_dict)?;

        // De-rendering.
        let mut derender = DerenderingEngine::new(boundary.clone(), metric3d.clone());
        let half = self.radius;
        let _ = derender.derender_region((-half, half), (-half / 2.0, half / 2.0), (-half / 2.0, half / 2.0));
        let g00_before: Vec<f64> =
            (0..n3).map(|i| derender.metric_calculator.metric_4d_grid[[i, mid3, mid3, 0, 0]]).collect();
        let g00_after: Vec<f64> =
            (0..n3).map(|i| derender.projected_metric[[i, mid3, mid3, 0, 0]]).collect();
        let derender_dict = PyDict::new(py);
        derender_dict.set_item("x_m", vec_to_pylist(py, &x3)?)?;
        derender_dict.set_item("g00_before", vec_to_pylist(py, &g00_before)?)?;
        derender_dict.set_item("g00_after", vec_to_pylist(py, &g00_after)?)?;
        derender_dict.set_item(
            "transferred_bits",
            derender.n_local_bits,
        )?;

        // Time-stepped re-rendering trajectory for the de-rendered region.
        let trajectory = derender.rerendering_trajectory(0.1, 1.0, 100.0);
        derender_dict.set_item("rerender_trajectory", vec_dict_to_pydict(py, &trajectory)?)?;
        out.set_item("derender", derender_dict)?;

        // Transient excitation and entropy-debt trajectory.
        let transient_engine = TransientRateEngine::new(1.0, 100.0, 0.1);
        let transient = transient_engine.run(&boundary, &engine, self.radius);
        let transient_dict = vec_dict_to_pydict(py, &transient)?;
        transient_dict.set_item("velocity_m_per_locktime", 0.1)?;
        out.set_item("transient", transient_dict)?;

        // Thermodynamics.
        let thermo = ThermodynamicRateEngine::new();
        let thermo_dict = PyDict::new(py);
        thermo_dict.set_item("steady_state_entropy_debt", thermo.steady_state_entropy_debt(POWER_BENCHMARK_MW))?;
        thermo_dict.set_item("kappa_per_joule", thermo.kappa_per_joule)?;
        thermo_dict.set_item("relaxation_rate_s_inv", thermo.relaxation_rate_s_inv)?;
        thermo_dict.set_item("maximum_hold_time_s", thermo.maximum_hold_time(0.0))?;
        thermo_dict.set_item("audit", audit_to_pydict(py, &thermo.audit())?)?;
        out.set_item("thermodynamics", thermo_dict)?;

        // Causal observer / power.
        let observer = CausalObserver::new(fg);
        let observer_audit = observer.audit(tolerance);
        let causal_dict = PyDict::new(py);
        let power_mw = CausalObserver::power_requirement_mw(self.radius);
        causal_dict.set_item("power_requirement_mw", power_mw)?;
        causal_dict.set_item("v_eff_c", (DELTA_MOD / 2.0).exp())?;
        causal_dict.set_item("audit", audit_to_pydict(py, &observer_audit)?)?;
        out.set_item("causal", causal_dict)?;

        out.set_item("power_mw", power_mw)?;
        out.set_item("v_eff_c", (DELTA_MOD / 2.0).exp())?;
        out.set_item("c_dark_residual", C_DARK_RESIDUAL)?;
        out.set_item("c_dark", C_DARK_COMP)?;
        out.set_item("delta_mod", DELTA_MOD)?;
        out.set_item("radius", self.radius)?;
        out.set_item("domain_radius", self.domain_radius)?;
        out.set_item("grid_points", self.grid_points)?;
        out.set_item("wall_steepness", self.wall_steepness)?;
        out.set_item("phase", self.phase)?;
        out.set_item("lepton_lift", boundary.parent_level as f64 / (2.0 * boundary.lepton_level as f64))?;
        out.set_item("quark_lift", boundary.parent_level as f64 / (3.0 * boundary.quark_level as f64))?;
        out.set_item("z_boundary", boundary.z_boundary)?;
        out.set_item("shannon_entropy", boundary.shannon_entropy)?;

        out.set_item("closure_norm", engine.closure_norm())?;
        out.set_item("unitarity_error", engine.audit(tolerance).get("unitarity_error").copied().unwrap_or(0.0))?;
        out.set_item("population_shift_l1", population_shift_l1)?;
        out.set_item("minimum_abs_metric_determinant", metric3d_audit.get("minimum_abs_determinant").copied().unwrap_or(0.0))?;
        out.set_item("minimum_gram_eigenvalue", metric3d_audit.get("minimum_gram_eigenvalue").copied().unwrap_or(0.0))?;
        out.set_item("observer_metric_error", observer_audit.get("observer_metric_error").copied().unwrap_or(0.0))?;
        out.set_item("acceleration_norm_m_s2", observer_audit.get("acceleration_norm_m_s2").copied().unwrap_or(0.0))?;
        out.set_item("power_scale_radius_m", power_scale_radius_m())?;
        out.set_item("n_sat_bits", N_SAT_BITS)?;
        out.set_item("n_local_bits", N_LOCAL_BITS_10M * (self.radius / DEFAULT_BUBBLE_RADIUS_M).powi(2))?;

        Ok(out)
    }
}

#[pymodule(name = "_core")]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Simulation>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_residual_matches_theory() {
        let expected = 834_433.0 / 362_670.0;
        assert!((C_DARK_RESIDUAL - expected).abs() < 1.0e-12);
    }

    #[test]
    fn boundary_framing_defect_is_zero() {
        let boundary = BoundaryRegister::new();
        let engine = ExcitationEngine::new(boundary, DEFAULT_PHASE_THETA);
        assert!(engine.framing_defect().abs() < 1.0e-12);
    }

    #[test]
    fn fg_slice_v_eff_c_matches_delta_mod() {
        let fg = FGSliceProjector::new(
            DELTA_MOD,
            DEFAULT_BUBBLE_RADIUS_M,
            DEFAULT_DOMAIN_RADIUS_M,
            DEFAULT_GRID_POINTS,
            DEFAULT_WALL_STEEPNESS_PER_M,
        );
        let expected = (DELTA_MOD / 2.0).exp();
        assert!((fg.v_eff_c() - expected).abs() < 1.0e-12);
    }

    #[test]
    fn causal_power_matches_benchmark() {
        assert!((CausalObserver::power_requirement_mw(DEFAULT_BUBBLE_RADIUS_M) - POWER_BENCHMARK_MW).abs() < 1.0e-10);
    }

    #[test]
    fn metric3d_audit_passes() {
        let metric = Metric3DCalculator::new(
            DEFAULT_BUBBLE_RADIUS_M,
            DEFAULT_DOMAIN_RADIUS_M,
            DEFAULT_STRESS_GRID_POINTS,
            DEFAULT_WALL_STEEPNESS_PER_M,
            DELTA_MOD,
        );
        let audit = metric.audit(1.0e-12);
        assert_eq!(audit["passed"], 1.0);
    }

    #[test]
    fn stress_energy_geometry_is_finite() {
        let metric = Metric3DCalculator::new(
            DEFAULT_BUBBLE_RADIUS_M,
            DEFAULT_DOMAIN_RADIUS_M,
            DEFAULT_STRESS_GRID_POINTS,
            DEFAULT_WALL_STEEPNESS_PER_M,
            DELTA_MOD,
        );
        let auditor = StressEnergyAuditor::new(
            metric.metric_4d_grid.clone(),
            (metric.x_m.clone(), metric.y_m.clone(), metric.z_m.clone()),
            LAMBDA_HOLO_SI,
            GRAVITATIONAL_CONSTANT_SI,
        );
        let audit = auditor.audit_energy_conditions(100, 26_008_312, 1.0e-12);
        assert_eq!(audit["passed"], 1.0);
        assert!(audit["maximum_null_norm_residual"] < 1.0e-6);
    }
}
