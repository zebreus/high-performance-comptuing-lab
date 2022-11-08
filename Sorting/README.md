---
title:  Exercise Two - Sorting
author: Prof. Dr. Ronald C. Moore 
        <ronald.moore@h-da.de>
date:   October 2021
...

# Exercise Two : Sorting

The goal of this exercise is to investigate ways to do sorting in parallel, using multiple hosts and message passing (!). 

### Code provided to Get Started

We will be sorting 100 byte records taken from files. This idea was taken from the "Sort Benchmark" hosted at <http://sortbenchmark.org/> -- see that web page for much more information.

There are three tools here:
1. **gensort** -- a tool for generating data to be sorted. This code is from <http://www.ordinal.com/gensort.html>.
2. **valsort** -- a tool for validating sorted output. This code is also from <http://www.ordinal.com/gensort.html>
3. **dumbSort** -- a non-parallel (single-threaded), very simple sorting program that takes input created by gensort and creates output which can be validated by valsort. This code was written by Prof. R. C. Moore (<ronald.moore@h-da.de>) for the HPC (formerly P&DC) course.

To build the tools, simply enter "make" on the command line. 

The first two tools (gensort and valsort) are built in the directory `gensort-1.5`. You should use these unchanged (unless you have *very* good reasons for changing either of them). The original version of this software is available at <http://www.ordinal.com/gensort.html>. The readme file and the license are also original. Build these tools by simply typing "make" on the command line.

The third tool (`dumbSort`) is a very simple C++ implementation that uses the standard C++ vector to store records and the standard C++ `sort` to sort them (and the standard C++ clock functions to report the time spent in input, sorting and output).

### Example Usage

To create a binary file called four.dat with 4 records (400 bytes long), run (from the command line)

    ./gensort 4 /tmp/four.dat
    
Note that the file is in the local "tmp" directory, so that we are not cluttering up our home directory. You can check that this file is not sorted (yet) by calling

    ./valsort /tmp/four.dat
    
To create a new file with the same 4 records in *sorted* order, call

    ./dumbSort /tmp/four.dat /tmp/sortedFour.dat
    
You can then validate this file by calling

    ./valsort /tmp/sortedFour.dat
    
When we're done, it is simply polite to delete our data files:

    rm -iv /tmp/*.dat

Notes: 
* Running "`make clean`" only cleans dumbsort and its intermediate files;
  it does not clean the `gensort` and `valsort` programs or the files in gensort-1.5.
  To really start over from a clean slate, run "`make distclean`".
  See the Makefiles (there are two) for exact details.
* Keeping your intermediate data in `/tmp` (as done above) is _not_ a good idea for message passage programs,
  where `/tmp` is a different directory on every machine (!).
  
### Exercise 2 -- Less Dumb Sorting

Your job now is to create a parallel, message passing, sort program. You may start with the code from the dumbsort program if you like, or you might want to start from scratch. This is up to you. 

Tip: You can find large (read-only) test input at the GSI in the directory 

    /lustre/hdahpc/datasets/Sorting/ 

After you have your own program up and running, start measuring its performance and write a lab report, as described in Moodle <https://lernen.h-da.de>.

Prof. R. C. Moore
