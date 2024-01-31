---
title:  Exercise One - Matrix Multiplication
author: Prof. Dr. Ronald C. Moore 
        <ronald.moore@h-da.de>
date:   Octorber 2021
...

Exercise 1 -- Matrix Multiplication
===================================

This directory contains a Makefile and two programs:
1. matrix_generator -- creates matrix files with random numbers. Output is written to standard out.
1. matrix_sequential -- multiplies two matrix files (in the format created by matrix_generator).


### Building the Programs

Enter "make" at the command line.

Enter "make test" to test both programs (generate very small matrices and then multiply them).

Enter "make clean" or "make distclean" to clean up.

### Running the programs

Run `make test` for a sample run of both programs.

For larger matrices, consider this example

```shell
$> ./matrix_generator 2000 2000 >matrix-2e3x2e3.txt # generates a 2000x2000 matrix
$> ./matrix_sequential matrix-2e3x2e3.txt matrix-2e3x2e3.txt >matrix-2e3squared.txt
setup time = 0.811296 seconds.
calculation time = 11.40617 seconds.
output time = 0.958524 seconds.
Total wall clock time = 13.17599 seconds.
$>
```

Note that `matrix_sequential` sends timing information to "standard erro", 
so if you redirect "standard output", you get a clean matrix file you can reuse,
without the timing data (which goes to the terminal).

To send the timing data to a second file, you can use (for example):

```shell
$> ./matrix_sequential matrix-A.txt matrix-B.txt >matrix-AxB.txt 2>timing.AxB.txt
$> cat timing.AxB.txt  # dumps the contents of timing.AxB.txt to the terminal
setup time = 0.000522 seconds.
calculation time = 5.8e-05 seconds.
output time = 9e-05 seconds.
Total wall clock time = 0.00067 seconds.
$>
```


### File Format


The file format is very simple.
The first line in each file contains the size of the matrix, i.e.

    numRows numColums

e.g.

    20 30

After that, each row contains one row of the matrix. The matrices hold floating point numbers, and all data is in text.

### Exercise

Write a better matrix multiplication program using threads and shared memory.
First, use OpenMP. This would suffice to pass this exercise.
For the highest number of points, consider how to use native threads.

### Acknowledgements and Apologies

This code is very old.
(Working with legacy code is part of being a computer scientist!).

It has been written over the years by Prof. Moore and Dr. Andreas Kirsch,
who has supervised the lab several times over the years (but no longer
works at the FbI/h_da (unfortunately)).

Prof. R. Moore <ronald.moore@h-da.de> now takes full responsibility for
maintaining the code you have received
(you are of course responsible for your copy of the code once you have received it).
