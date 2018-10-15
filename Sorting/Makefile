# This Makefile is brutally simple, not very sophisticated,
# and not at all up to my normal standards...
# ...Be grateful, dear reader!
# (Signed, Prof. R. Moore, h_da, fbi )

# --------------------------------------------------------------------------
# Alternatives Compilers:
# CC = g++7
# CC = g++8
CC = g++
# CC = clang++ 
# CC = c++

CFLAGS=-Wall -O3

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




