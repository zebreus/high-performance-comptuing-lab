/* -*- Mode: C++; c-basic-offset:4 ; -*- */
/*  This code borrows heavily from the file mpi-c from the
 *  Argonne National Laboratory.
 */

#include <iostream>
#include <iomanip>
#include <sstream>
#include <string>
#include <chrono>
// uncomment to disable assert()
// #define NDEBUG
#include <cassert>
#include <cmath> // for fabs()

#include <omp.h>

// ******************* Utilities
int string2int( const std::string &text  )
{
    std::stringstream    temp( text );
    int     result = 0xffffffff;
    temp >> result;
    return result;
} // end string2int


// ******************* Functions for calculating PI (borrowed from mpi)
double f(double a)
{
    return (4.0 / (1.0 + a*a));
} // end f

const double PI25DT = 3.141592653589793238462643; // No, we're not cheating -this is for testing!
const long   n = 42l * 1024 * 1024;               // default # of rectangles (42l = long int 42)
const double h   = 1.0 / (double) n;

const double maxNumThreads = 1024; // this is only for sanity checking

// ******************* main
int main(int argc,char *argv[])
{
    int numThreads = 0;

    if ( 2 == argc )
        numThreads = string2int( argv[ 1 ] );
    else // if number of args illegal
    {
        std::cerr << "Usage: " << argv[0] << " number-of-threads" << std::endl;
        return( -1 );
    }; // end argc check

    assert( 0 < numThreads );
    assert( numThreads <= maxNumThreads );

    std::chrono::system_clock::time_point startTime = std::chrono::system_clock::now();

    double sum = 0;
    
    //  *** Here is the OpenMP Magic!! (All in one line) ***
    #pragma omp parallel for num_threads( numThreads ) reduction(+:sum)
    for ( long i = 1; i <= n; i += 1 )
    {
        double x = h * ((double)i - 0.5);
        sum += f(x);
    }
    /* It would have been better to start from large i and count down, by the way. */

    double pi = h * sum;

    std::chrono::system_clock::time_point endTime = std::chrono::system_clock::now();
    std::chrono::microseconds microRunTime
         = std::chrono::duration_cast<std::chrono::microseconds>(endTime - startTime);
    double runTime = microRunTime.count() / 1000000.0;

    std::cout << std::setprecision( 16 )
              << "Pi is approximately " << pi
              << ", Error is " << std::fabs(pi - PI25DT) << std::endl;
    std::cout << std::setprecision( 8 )
              << "Wall clock time = " << runTime  << " seconds."
              << std::endl;
    std::cout << "There were " << numThreads << " threads." << std::endl;

    return 0;
}
