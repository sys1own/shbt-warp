.PHONY: all test sim cad paper clean

all: test sim cad paper

.venv:
	python3 -m venv .venv
	. .venv/bin/activate && pip install -q -r requirements.txt

rust: .venv
	. .venv/bin/activate && maturin develop --release

test: rust
	. .venv/bin/activate && pytest

sim: rust
	. .venv/bin/activate && python -m shbt_warp.cli --figures-directory figures --tex-output sim_results.tex

cad: rust
	. .venv/bin/activate && shbt-cad-sim --figures-directory figures --tex-output cad_sim_results.tex

paper: sim cad
	TEXINPUTS=.:./sections//: pdflatex -interaction=nonstopmode main.tex
	TEXINPUTS=.:./sections//: pdflatex -interaction=nonstopmode main.tex

clean:
	rm -rf figures/*.pdf sim_results.tex cad_sim_results.tex sweep_results.csv mainNotes.bib *.aux *.log *.out *.toc *.synctex.gz
