# gensort and valsort Makefile
#
#  This version has been simplified by R. Moore for use at the 
# h_da. Target "clean" has also been added.

USE_SUMP_PUMP=0
# use BITS=-m32 to force 32-bit build on a 64-bit platform
BITS=
CFLAGS=$(BITS) -O3 -Wall

all: gensort valsort

gensort: gensort.c rand16.o rand16.h
	cc -g $(CFLAGS) -o gensort gensort.c rand16.o -lz

valsort: valsort.c rand16.o rand16.h
	cc -g $(CFLAGS) -o valsort valsort.c rand16.o -lz

rand16.o: rand16.c rand16.h
	cc -g $(CFLAGS) -c rand16.c

clean:
	-rm -f *.o gensort valsort

