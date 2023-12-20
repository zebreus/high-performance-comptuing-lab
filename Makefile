PWD = $(shell pwd)
ASCIIDOCTOR     = asciidoctor -r asciidoctor-kroki
ASCIIDOCTOR_WEB_PDF = asciidoctor-web-pdf -r asciidoctor-kroki
PYTHON = python3

TARGETS += $(TARGETS_WITHOUT_HTML) index.html
TARGETS_WITHOUT_HTML += $(PROCESSED_CHARTS) $(wildcard scripts/*.adoc)

VEGA_CHART_FILES = $(shell find assets -name '*.vl.json')
VEGA_DATA_FILES = $(addprefix assets/,$(shell grep -Poh "[^\"]+.csv" $(VEGA_CHART_FILES) /dev/null | sort | sed 's/^\.\///' | uniq))
PROCESSED_CHARTS = $(addprefix processed-assets/,$(notdir $(VEGA_CHART_FILES)))

# .EXTRA_PREREQS:=Makefile
.PHONY: all pdf preview
all: paper.pdf
preview: paper-preview

.PRECIOUS: KickOff/results_gcc.csv KickOff/results_clang.csv

KickOff/results_gcc.csv:
	cd KickOff && make clean
	cd KickOff && make CXX=g++ CC=gcc
	cd KickOff && make benchmark
	cp KickOff/results.csv KickOff/results_gcc.csv

KickOff/results_clang.csv:
	cd KickOff && make clean
	cd KickOff && make CXX=clang++ CC=clang
	cd KickOff && make benchmark
	cp KickOff/results.csv KickOff/results_clang.csv

KICKOFF_DATA_TARGETS = assets/compiler-comparison.csv assets/mpi-cpp.csv assets/implementation-comparison-fixed-n.csv assets/implementation-comparison-fixed-threads.csv
SORTING_DATA_TARGETS = assets/sorting-duration-on-one-node.csv

$(KICKOFF_DATA_TARGETS) : KickOff/results_gcc.csv KickOff/results_clang.csv scripts/collect_data.ts
	deno run -A scripts/collect_data.ts

$(SORTING_DATA_TARGETS) : Sorting/results.csv scripts/collect_sorting_data.ts
	deno run -A scripts/collect_sorting_data.ts

data-kickoff: $(KICKOFF_DATA_TARGETS)
data-sorting: $(SORTING_DATA_TARGETS)

SCSS_FILES = $(wildcard styles/*.scss) $(wildcard styles/*/*.scss) $(wildcard styles/*/*/*.scss)

paper.css: $(SCSS_FILES)
	cd styles ; sass --update --sourcemap=none paper.scss:../paper.css

%.html: %.adoc paper.css $(TARGETS_WITHOUT_HTML) $(PROCESSED_CHARTS)
	$(ASCIIDOCTOR) -S unsafe $< -o $@

%-web-preview: %.html
	$(PYTHON) scripts/serve.py $<

%.pdf: %.adoc paper.css $(TARGETS_WITHOUT_HTML) $(PROCESSED_CHARTS)
	$(ASCIIDOCTOR_WEB_PDF) $< -o $@

%-preview: %.adoc paper.css $(TARGETS_WITHOUT_HTML) $(PROCESSED_CHARTS)
	$(ASCIIDOCTOR_WEB_PDF) --preview $<

# Preprocess vega charts
process-charts: $(PROCESSED_CHARTS)

$(PROCESSED_CHARTS) : processed-assets/%.vl.json : assets/%.vl.json $(VEGA_DATA_FILES)
	mkdir -p $(dir $@)
	bash scripts/process_chart.sh $< > $@
	touch $@

# data:
# 	cd experiments && make data
# 	make $(VEGA_DATA_FILES)

# $(ALL_DATA_JSON):
# 	cd experiments && make data

# $(VEGA_DATA_FILES) &: $(ALL_DATA_JSON) data/make_data.sh
# 	bash data/make_data.sh $< data

clean:
	rm -rf report1.pdf $(VEGA_DATA_FILES) processed-assets

dist-clean: clean
	rm -rf $(TARGETS)

.PHONY: clean all dist-clean process-charts data