/* -*- Mode: C++; c-basic-offset:4 ; -*- */
/*  This code borrows heavily from the file mpi-c from the
 *  Argonne National Laboratory.
 */

#include <chrono>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>
#include <thread>
#include <vector>
// uncomment to disable assert()
// #define NDEBUG
#include <cassert>
#include <cmath>  // for fabs()

// ******************* Utilities
int string2int(const std::string& text) {
    std::stringstream temp(text);
    int result = 0xffffffff;
    temp >> result;
    return result;
}  // end string2int

// ******************* Functions for calculating PI (borrowed from mpi)
double f(double a) {
    return (4.0 / (1.0 + a * a));
}  // end f

const double PI25DT =
    3.141592653589793238462643;  // No, we're not cheating -this is for testing!

const double maxNumThreads = 1024;  // this is only for sanity checking

long n;
double h;

void pi_thread(int thread_num, int numThreads, double* partial_pi) {
    assert(0 <= thread_num);
    assert(thread_num < maxNumThreads);
    double sum = 0.0;
    /* It would have been better to start from large i and count down, by the way. */
    for (long i = thread_num + 1; i <= n; i += numThreads) {
        double x = h * ((double)i - 0.5);
        sum += f(x);
    }
    *partial_pi = h * sum;

}  // end pi_thread

// ******************* main
int main(int argc, char* argv[]) {
    int numThreads = 0;

    if (3 == argc) {
        numThreads = string2int(argv[1]);
        n = string2int(argv[2]) * 1024l * 1024l;
        h = 1.0 / (double)n;
    } else  // if number of args illegal
    {
        std::cerr << "Usage: " << argv[0] << " number-of-threads n" << std::endl;
        return (-1);
    };  // end argc check

    assert(0 < numThreads);
    assert(numThreads <= maxNumThreads);

    std::chrono::system_clock::time_point startTime =
        std::chrono::system_clock::now();

    std::thread threads[numThreads];  // Note: No REAL threads yet...
    double partials[numThreads];

    //Launch the threads
    for (int i = 0; i < numThreads; ++i) {
        threads[i] = std::thread(pi_thread, i, numThreads, &(partials[i]));
    }

    ////Join the threads with the main thread
    double pi = 0;
    for (int i = 0; i < numThreads; ++i) {
        threads[i].join();
        pi += partials[i];
    }

    std::chrono::system_clock::time_point endTime =
        std::chrono::system_clock::now();
    std::chrono::microseconds microRunTime =
        std::chrono::duration_cast<std::chrono::microseconds>(
            endTime - startTime
        );
    double runTime = microRunTime.count() / 1000000.0;

    std::cout << std::setprecision(16) << "Pi is approximately " << pi
              << ", Error is " << std::fabs(pi - PI25DT) << std::endl;
    std::cout << std::setprecision(8) << "Wall clock time = " << runTime
              << " seconds." << std::endl;
    std::cout << "There were " << numThreads << " threads." << std::endl;
    std::cerr << runTime << std::flush;
    return 0;
}
