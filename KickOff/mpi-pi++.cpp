/* -*- Mode: C++; c-basic-offset:4 ; -*- */
/*  This code borrows heavily from the file mpi-c from the
 *  Argonne National Laboratory.
 */

#include <chrono>
#include <iomanip>
#include <iostream>
#include <fstream>
#include <sstream>
#include <string>
// uncomment to disable assert()
// #define NDEBUG
#include <mpi.h>

#include <cassert>
#include <cmath>  // for fabs()

// ******************* Functions for calculating PI (borrowed from mpi)
double f(double a) {
    return (4.0 / (1.0 + a * a));
}  // end f

const double PI25DT =
    3.141592653589793238462643;  // No, we're not cheating -this is for testing!

// ******************* Utilities
int string2int(const std::string& text) {
    std::stringstream temp(text);
    int result = 0xffffffff;
    temp >> result;
    return result;
}

// ******************* main
int main(int argc, char* argv[]) {
    /** Standard MPI opening boilerplate **/
    int myid, numprocs, namelen;
    char processor_name[MPI_MAX_PROCESSOR_NAME];
    MPI_Init(&argc, &argv);
    MPI_Comm_size(MPI_COMM_WORLD, &numprocs);
    MPI_Comm_rank(MPI_COMM_WORLD, &myid);
    MPI_Get_processor_name(processor_name, &namelen);

    std::cout << "Process " << myid << " of " << numprocs << ", running on "
              << processor_name << std::endl;

    /** Actual work starts here */

    double startwtime = 0.0;
    // Strictly speaking, we could have made n a constant, like it is in our other programs,
    // but this way we can demonstrate how a broadcast works.
    long n;  // n is the number of rectangles
    if (0 == myid) {
        startwtime = MPI_Wtime();
        n = strtol(argv[1], nullptr, 10) * 1024l * 1024l;
    };

    MPI_Bcast(
        &n,
        1,
        MPI_LONG_INT,  // Broadcast n, which is 1 long integer, where
        0,
        MPI_COMM_WORLD
    );  // ID 0 sends & everyone else in COMM_WORLD receives.

    double h = 1.0 / (double)n;
    double sum = 0;

    /* It would have been better to start from large i and count down, by the way. */
    for (long i = myid + 1; i <= n; i += numprocs) {
        double x = h * ((double)i - 0.5);
        sum += f(x);
    }

    double pi, mypi = h * sum;
    MPI_Reduce(
        &mypi,
        &pi,
        1,
        MPI_DOUBLE,
        MPI_SUM,  // Everyone's copy of mypi (one double) are summed up,
        0,
        MPI_COMM_WORLD
    );  // and the result is sent to ID 0, and stored in pi.

    if (0 == myid) {
        double endwtime = MPI_Wtime();
        std::cout << std::setprecision(16) << "Pi is approximately " << pi
                  << ", Error is " << fabs(pi - PI25DT) << std::endl;
        std::cout << "Wall clock time = " << (endwtime - startwtime)
                  << " seconds." << std::endl;
        std::cout << "There were " << numprocs << " processes." << std::endl;

        std::cerr << (endwtime - startwtime) << std::flush;
    }



    MPI_Finalize();

    return 0;
}
