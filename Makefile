.PHONY: all sim paper clean

all: sim paper

sim:
	pip install -r requirements.txt
	maturin develop --release
	shbt-warp-sim --figures-directory figures --tex-output sim_results.tex

paper: sim
	pdflatex main.tex
	pdflatex main.tex

clean:
	rm -rf figures/*.pdf sim_results.tex *.aux *.log *.out *.toc
