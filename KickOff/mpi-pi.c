/* -*- Mode: C; c-basic-offset:4 ; -*- */
/*
 *  (C) 2001 by Argonne National Laboratory.
 *      See COPYRIGHT in top-level directory.
 */

#include <mpi.h>
#include <stdio.h>
#include <math.h>

double f(double a)
{
    return (4.0 / (1.0 + a*a));
}

int main(int argc,char *argv[])
{
    // Strictly speaking, we could have made n a constant, like it is in our other programs, 
    // but this way we can demonstrate how a broadcast works.
    long   i, n;    // n is the number of rectangles, i is the rectangle number.
    int    myid, numprocs;
    const double PI25DT = 3.141592653589793238462643;
    double mypi, pi, h, sum, x;
    double startwtime = 0.0, endwtime;
    int    namelen;
    char   processor_name[MPI_MAX_PROCESSOR_NAME];

	/** Standard MPI opening boilerplate **/
    MPI_Init(&argc,&argv);
    MPI_Comm_size(MPI_COMM_WORLD,&numprocs);
    MPI_Comm_rank(MPI_COMM_WORLD,&myid);
    MPI_Get_processor_name(processor_name,&namelen);

    fprintf(stdout,"Process %d of %d is on %s\n",
        myid, numprocs, processor_name);
    fflush(stdout);

    /** Actual work starts here */

    if (0 == myid) {
    	startwtime = MPI_Wtime();    	
    	n = 42l * 1024 * 1024; /* default # of rectangles (42l = long int 42) */
    };

    MPI_Bcast(&n, 1, MPI_LONG_INT, 0, MPI_COMM_WORLD);

    h   = 1.0 / (double) n;
    sum = 0.0;
    /* It would have been better to start from large i and count down, by the way. */
    for (i = myid + 1; i <= n; i += numprocs)
    {
		x = h * ((double)i - 0.5);
		sum += f(x);
    }
    mypi = h * sum;

    MPI_Reduce(&mypi, &pi, 1, MPI_DOUBLE, MPI_SUM, 0, MPI_COMM_WORLD);

    if (0 == myid) {
		endwtime = MPI_Wtime();
		printf("Pi is approximately %.16f, Error is %.8e\n",
			   pi, fabs(pi - PI25DT));
		printf("Wall clock time = %.8f seconds.\n", (endwtime-startwtime) );
		fflush(stdout);
    }

    MPI_Finalize();
    return 0;
}
