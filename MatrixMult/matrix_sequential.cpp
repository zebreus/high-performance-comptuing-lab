// file...: matrix_sequential.cpp
// desc...: The reference solution for sequential matrix multiplication
// oct-2010 | a.knirsch@fbi.h-da.de
// oct-2021 | Major Revision -- Simplified CMatrix class (!) | ronald.moore@h-da.de

#include <stdlib.h>  // for exit()

#include <chrono>
#include <iomanip>  // for std::setprecision
#include <iostream>

#include "CMatrix.h"
#include "cxxopts.hpp"

void errorExit(const char* progname, const char* error) {
    if (error != NULL) {
        std::cerr << "ERROR: " << error << std::endl;
    }
    std::cerr
        << "usage: " << progname << " <matrix1> <matrix2>" << std::endl
        << "\twhere <matrix1> and <matrix2> are file names containing matrices."
        << std::endl;
    exit(EXIT_FAILURE);
}

inline double secondsSince(
    std::chrono::time_point<std::chrono::system_clock> alpha,
    std::chrono::time_point<std::chrono::system_clock> omega
) {
    auto delta =
        std::chrono::duration_cast<std::chrono::microseconds>(omega - alpha);
    double result = delta.count() / 1000000.0;
    return result;
}

// tag::algorithm[]
CMatrix multiply(CMatrix const& m1, CMatrix const& m2) {
    CMatrix result(m2.width, m1.height);  // allocate memory
    for (unsigned int row = 0; row < m1.height; row++) {
        for (unsigned int col = 0; col < m2.width; col++) {
            double sum = 0.0;
            for (unsigned int dot = 0; dot < m2.height; dot++) {
                sum += m1[row][dot] * m2[dot][col];
            }
            result[row][col] = sum;
        }
    }
    return result;
}

// end::algorithm[]

// +++ main starts here +++
int main(int argc, char** argv) {
    cxxopts::Options options(
        "matrix_sequential",
        "Multiply two matrices in C++ without parallelization"
    );
    options.add_options()
        ("a,matrix-a", "File containing the first matrix", cxxopts::value<std::string>())
        ("b,matrix-b", "File containing the second matrix", cxxopts::value<std::string>())
        ("t,threads", "Only provided for compatibility", cxxopts::value<unsigned int>()->default_value("1"))
        ("h,help", "Print usage")
    ;

    options.parse_positional({"matrix-a", "matrix-b"});
    auto parsedOptions = options.parse(argc, argv);
    if (parsedOptions.count("help")) {
        std::cout << options.help() << std::endl;
        exit(0);
    }

    auto numberOfThreads = parsedOptions["threads"].as<unsigned int>();
    auto pathMatrixA = parsedOptions["matrix-a"].as<std::string>();
    auto pathMatrixB = parsedOptions["matrix-b"].as<std::string>();

    if (numberOfThreads != 1) {
        errorExit(
            argv[0],
            "matrix_sequential does only support a single thread."
        );
    }

    auto startTime = std::chrono::system_clock::now();

    CMatrix m1(pathMatrixA.data());  // read 1st matrix
    CMatrix m2(pathMatrixB.data());  // read 2nd matrix

    if (m1.width != m2.height)  // check compatibility
        errorExit(
            argv[0],
            "Width of 1st matrix not equal to height of 2nd matrix."
        );

    auto milestoneSetup = std::chrono::system_clock::now();
    std::cerr << std::setprecision(8)
              << "setup time = " << secondsSince(startTime, milestoneSetup)
              << " seconds." << std::endl;
    // We dont want to measure the printing time
    milestoneSetup = std::chrono::system_clock::now();

    auto result = multiply(m1, m2);

    auto milestoneCalculate = std::chrono::system_clock::now();
    std::cout << std::fixed << std::setprecision(12)
              << secondsSince(milestoneSetup, milestoneCalculate) << ","
              << result.value() << std::endl;
    std::cerr << std::fixed << std::setprecision(8) << "calculation time = "
              << secondsSince(milestoneSetup, milestoneCalculate) << " seconds"
              << std::endl;

    std::cerr << "sum of the result: " << result.value() << std::endl;
    return 0;
}

// EOF
