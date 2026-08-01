"""Matplotlib vector PDF figure generator."""

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path


def _save_figure(fig, path):
    """Save a figure to a vector PDF preserving all text bounding boxes."""
    fig.savefig(path, format="pdf", bbox_inches="tight")


class PlotGenerator:
    """Render the standard SHBT warp simulation figures."""

    def __init__(self, results):
        self.results = results

    def render_all(self, figures_dir):
        """Generate all figures used by the manuscript and audit plots."""
        figures_dir = Path(figures_dir)
        figures_dir.mkdir(parents=True, exist_ok=True)
        self._warp_bubble_profile(figures_dir / "warp_bubble_profile.pdf")
        self._shift_profile(figures_dir / "shift_profile.pdf")
        self._stress_energy_audit(figures_dir / "stress_energy_audit.pdf")
        self._derendering_transition(figures_dir / "derendering_transition.pdf")
        self._entropy_gradient(figures_dir / "entropy_gradient.pdf")

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
        ax1.plot(x, shape, "b-", lw=1.2, label=r"$f_{\mathrm{SHBT}}(x)$")
        ax1.set_xlabel("x (m)")
        ax1.set_ylabel("Shape function", color="b")
        ax1.tick_params(axis="y", labelcolor="b")

        ax2 = ax1.twinx()
        ax2.plot(x, beta, "r--", lw=1.2, label=r"$\beta_x$ (m/s)")
        ax2.set_ylabel("Shift vector $\\beta_x$ (m/s)", color="r")
        ax2.tick_params(axis="y", labelcolor="r")

        fig.suptitle("Warp bubble profile")
        fig.tight_layout()
        _save_figure(fig, path)
        plt.close(fig)

    def _shift_profile(self, path):
        fg = self.results.get("fg_slice", {})
        x = self._to_array(fg.get("x_m", []))
        beta_over_c = self._to_array(fg.get("beta_over_c", []))

        fig, ax = plt.subplots(figsize=(7, 4))
        ax.plot(x, beta_over_c, "r-", lw=1.2)
        ax.axhline(-fg.get("v_eff_c", 0.0), color="k", ls=":", lw=0.8, label=r"$-v_{\mathrm{eff}}/c$")
        ax.set_xlabel("x (m)")
        ax.set_ylabel(r"$\beta_x / c$")
        ax.set_title("Shift profile")
        ax.legend()
        fig.tight_layout()
        _save_figure(fig, path)
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
        ax2.plot(x, t00, "m--", lw=1.2, label=r"$T_{00}^{\mathrm{eff}}$")
        ax2.set_ylabel(r"Effective $T_{00}$", color="m")
        ax2.tick_params(axis="y", labelcolor="m")

        fig.suptitle("Stress-energy audit (center line)")
        fig.tight_layout()
        _save_figure(fig, path)
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
        _save_figure(fig, path)
        plt.close(fig)

    def _entropy_gradient(self, path):
        boundary = self.results.get("boundary", {})
        excitation = self.results.get("excitation", {})
        base = np.array(self._to_array(boundary.get("shannon_density", []))).reshape(3, 3)
        excited = np.array(self._to_array(excitation.get("excited_shannon_contributions", []))).reshape(3, 3)

        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(9, 4))
        for ax, data, title, cmap in [
            (ax1, base, "Baseline Shannon density", "viridis"),
            (ax2, excited, "Excited Shannon contributions", "plasma"),
        ]:
            im = ax.imshow(data, cmap=cmap, origin="upper", extent=(-0.5, 2.5, 2.5, -0.5))
            ax.set_xticks([0, 1, 2])
            ax.set_yticks([0, 1, 2])
            ax.set_xlabel("SU(3) weight index")
            ax.set_ylabel("SU(2) charge index")
            ax.set_title(title)
            fig.colorbar(im, ax=ax, shrink=0.7)

        # Overlay discrete gradient arrows on the excited panel.
        gy, gx = np.gradient(excited)
        X, Y = np.meshgrid(np.arange(3), np.arange(3))
        ax2.quiver(X, Y, gx, -gy, color="w", scale=0.05, width=0.015)

        fig.suptitle(rf"Shannon entropy gradient at $\theta={self.results.get('phase', 0.421)}$", y=0.98)
        fig.tight_layout()
        fig.subplots_adjust(bottom=0.18, top=0.85, left=0.08, right=0.96, wspace=0.35)
        _save_figure(fig, path)
        plt.close(fig)
