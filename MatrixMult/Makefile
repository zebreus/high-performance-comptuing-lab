################################################################
## SIMPLE MAKEFILE TO BUILD WITH MPICH                        ##
################################################################
# oct-2010 - a.knirsch@fbi.h-da.de                             #
# April-2013 - ronald.moore@h-da.de  (added make test)         #
# Oct-2021 - Removed MPI stuff
################################################################

# Set CXX to be clang++ or g++ or mpic++ -- compare kickoff makefile.
# CXX	= c++  
# CXX = clang++
CXX = g++

CFLAGS	= -O3 --std=c++17 -Wall
PROGS	= matrix_sequential matrix_generator
OBJS	= matrix_sequential.o matrix_generator.o CMatrix.o
TESTFILES = matrix-A-10x10.txt matrix-B-10x10.txt

CFLAGS	+= -I/usr/local/include/
LDFLAGS += -L/usr/local/lib/

all: $(PROGS) $(TESTFILES)

#      "Variables", die hier benutzt werden:
#      Vgl. http://www.makelinux.net/make3/make3-CHP-2-SECT-2.html
# $@ = The filename representing the target.
# $< = The filename of the first prerequisite.
# $* = The stem of the filename of the target (i.e. without .o, .cpp...)
# $^ = The names of all the prerequisites, with spaces between them.

# This works but it recompiles too much ...
# $(PROGS): $(OBJS)
# 	$(CXX) $(LDFLAGS) -o $@  $@.o  CMatrix.o

# This is more accurate (and thus does less work)
matrix_sequential: matrix_sequential.o CMatrix.o
	$(CXX) $(LDFLAGS) -o $@  $@.o  CMatrix.o

matrix_generator:  matrix_generator.o CMatrix.o
	$(CXX) $(LDFLAGS) -o $@  $@.o  CMatrix.o

CMatrix.o: CMatrix.cpp CMatrix.h
	$(CXX) $(CFLAGS) -c -o $@ $<

%.o: %.cpp CMatrix.h
	$(CXX) $(CFLAGS) -c -o $@ $<

$(TESTFILES): matrix_generator
	./matrix_generator 10 10 >$@

clean:
	-rm -rf $(PROGS) *~ *.o ._*

distclean: clean
	-rm -rf $(TESTFILES)

test:  $(PROGS) $(TESTFILES)
	echo "Calling matrix_sequential..."
ifeq (, $(shell which srun)) # When srun is not available, assume we want to run locally
	./matrix_sequential $(TESTFILES)
else # if srun is available, use it -- e.g. at the GSI!
#      run with one process (-n 1) and one thread (-c 1) (on the debug partition)
	srun -p debug -n 1 -c 1 -- ./matrix_sequential $(TESTFILES)
endif

