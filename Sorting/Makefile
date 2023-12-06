# This Makefile is brutally simple, not very sophisticated,
# and not at all up to my normal standards...
# ...Be grateful, dear reader!
# (Signed, Prof. R. Moore, h_da, fbi )

# --------------------------------------------------------------------------
# Alternatives Compilers:
# CC = g++7
# CC = g++8
# CC = g++
CC = clang++ 
# CC = c++

# CFLAGS=-Wall -O3

# For recursive make in the UTILS subdirectory, see:
# https://www.gnu.org/software/make/manual/make.html#Recursion

UTILS=gensort-1.5

.PHONY: clean distclean utils

All: dumbSort utils

clean: 
	-rm -f *.o dumbSort

distclean: clean
	-rm -f *.o gensort valsort
	$(MAKE) -C $(UTILS) clean

utils:
	$(MAKE) -C $(UTILS)

gensort: $(UTILS)/gensort utils
	cp $(UTILS)/gensort .

valsort: $(UTILS)/valsort utils
	cp $(UTILS)/valsort .

dumbSort: dumbSort.cpp gray_sort_record.h
	$(CC) -std=c++11 $(CFLAGS) -o dumbSort dumbSort.cpp

test: dumbSort gensort valsort
	./gensort -a 10000000 testfile
	./dumbSort testfile testfile.sorted
	./valsort testfile.sorted

run-rust: 
	unset CC && cd rust-sorting && cargo run --release  -- --algorithm glidesort /mnt/toast/10gb.dat /mnt/toast/

run-cpp: dumbSort
	./dumbSort ./1gb.dat ./1gb.dat.sorted


run-mpi: 
	unset CC && cd rust-sorting && cargo build --release
	mpiexec -n 3 ../target/release/rust-sorting --algorithm mpi-block-sort 1gb.dat ./
	./valsort output.sorted

run-radix: 
	unset CC && cd rust-sorting && cargo build --release
	mpiexec --oversubscribe -n 16 ../target/release/rust-sorting --algorithm mpi-distributed-radix-sort /mnt/toast/10gb.dat /mnt/toast/
	./valsort /mnt/toast/output.sorted