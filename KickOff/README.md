---
title:  Exercise Zero - Gettig Started
author: Prof. Dr. Ronald C. Moore 
        <ronald.moore@h-da.de>
date:   Oct 2022
...




Exercise Zero : Getting Started
===============================

The goal of this exercise is to get up and running. 

You should find here four programs and one make file.

Before building (or testing) the three programs provided, _please_ read the `Makefile`. 
This file has many comments -- and many of these are lines that have been "commented out",
which represent alternatives. 
You will _probably_ want to choose different alternatives for _your_ machine, 
and you should _definitely_ choose different alternatives when testing at the GSI
(in order to use SLURM).

Once you have reviewed the `Makefile`:

  *  To build all of the programs, enter `make` on the command line.
  *  To test all of the programs, enter `make tests` on the command line.

The programs are provided in a state which runs _very_ quickly. 
This is good for establishing whether they run correctly. 
This is _not_ good for benchmarking performance!

The next step is to modify these programs so that they run **much** longer than they do now.

You will want to change the Makefile as well -- to use different numbers of threads.  
You should also consider converting the "tests" section of the `Makefile` into a shell script that can be run on its own (without `make`)

After you have your versions of the programs (and Makefile) up and running, start measuring their performance and write a lab report, as described in Moodle. 

Prof. R. C. Moore
