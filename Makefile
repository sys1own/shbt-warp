.PHONY: all sim paper clean

all: sim paper

.venv:
	python3 -m venv .venv
	. .venv/bin/activate && pip install -q -r requirements.txt

sim: .venv
	. .venv/bin/activate && maturin develop --release
	. .venv/bin/activate && python -m shbt_warp.cli --figures-directory figures --tex-output sim_results.tex

paper: sim
	TEXINPUTS=.:./sections//: pdflatex -interaction=nonstopmode main.tex
	TEXINPUTS=.:./sections//: pdflatex -interaction=nonstopmode main.tex

clean:
	rm -rf figures/*.pdf sim_results.tex mainNotes.bib *.aux *.log *.out *.toc
