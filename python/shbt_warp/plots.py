"""Matplotlib vector PDF figure generator."""

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from pathlib import Path


class PlotGenerator:
    """Render the three standard SHBT warp simulation figures."""

    def __init__(self, results):
        self.results = results

    def render_all(self, figures_dir):
        """Generate ``warp_bubble_profile.pdf``, ``stress_energy_audit.pdf``,
        and ``derendering_transition.pdf`` inside ``figures_dir``."""
        figures_dir = Path(figures_dir)
        figures_dir.mkdir(parents=True, exist_ok=True)
        self._warp_bubble_profile(figures_dir / "warp_bubble_profile.pdf")
        self._stress_energy_audit(figures_dir / "stress_energy_audit.pdf")
        self._derendering_transition(figures_dir / "derendering_transition.pdf")

    def _to_array(self, value):
        if hasattr(value, "tolist"):
            return value.tolist()
        return list(value)

    def _warp_bubble_profile(self, path):
        fg = self.results.get("fg_slice", {})
        x = self._to_array(fg.get("x_m", []))
        shape = self._to_array(fg.get("shape", []))
        beta = self._to_array(fg.get("beta_m_s", []))

        fig, ax1 = plt.subplots(figsize=(7, 4))
        ax1.plot(x, shape, "b-", lw=1.2, label=r"$f_{\\mathrm{SHBT}}(x)$")
        ax1.set_xlabel("x (m)")
        ax1.set_ylabel("Shape function", color="b")
        ax1.tick_params(axis="y", labelcolor="b")

        ax2 = ax1.twinx()
        ax2.plot(x, beta, "r--", lw=1.2, label=r"$\\beta_x$ (m/s)")
        ax2.set_ylabel("Shift vector $\\beta_x$ (m/s)", color="r")
        ax2.tick_params(axis="y", labelcolor="r")

        fig.suptitle("Warp bubble profile")
        fig.tight_layout()
        fig.savefig(path, format="pdf")
        plt.close(fig)

    def _stress_energy_audit(self, path):
        stress = self.results.get("stress_energy", {})
        x = self._to_array(stress.get("x_m", []))
        ricci = self._to_array(stress.get("ricci_scalar", []))
        t00 = self._to_array(stress.get("energy_density_t00", []))

        fig, ax1 = plt.subplots(figsize=(7, 4))
        ax1.plot(x, ricci, "g-", lw=1.2, label=r"$R$")
        ax1.set_xlabel("x (m)")
        ax1.set_ylabel("Ricci scalar", color="g")
        ax1.tick_params(axis="y", labelcolor="g")

        ax2 = ax1.twinx()
        ax2.plot(x, t00, "m--", lw=1.2, label=r"$T_{00}^{\\mathrm{eff}}$")
        ax2.set_ylabel(r"Effective $T_{00}$", color="m")
        ax2.tick_params(axis="y", labelcolor="m")

        fig.suptitle("Stress-energy audit (center line)")
        fig.tight_layout()
        fig.savefig(path, format="pdf")
        plt.close(fig)

    def _derendering_transition(self, path):
        derender = self.results.get("derender", {})
        x = self._to_array(derender.get("x_m", []))
        before = self._to_array(derender.get("g00_before", []))
        after = self._to_array(derender.get("g00_after", []))

        fig, ax = plt.subplots(figsize=(7, 4))
        ax.plot(x, before, "k-", lw=1.2, label=r"$g_{00}$ before")
        ax.plot(x, after, "c--", lw=1.2, label=r"$g_{00}$ after de-render")
        ax.set_xlabel("x (m)")
        ax.set_ylabel(r"$g_{00}$")
        ax.set_title("De-rendering transition (center line)")
        ax.legend()
        fig.tight_layout()
        fig.savefig(path, format="pdf")
        plt.close(fig)
