// file...: matrix_sequential.cpp
// desc...: minimalistic matrix multiplication solution
// oct-2010 | a.knirsch@fbi.h-da.de
// oct-2021 | Major Revision -- Simplified CMatrix class (!) | ronald.moore@h-da.de

#include <iostream>
#include <iomanip> // for std::setprecision
#include <chrono>
#include <stdlib.h> // for exit()

#include "CMatrix.h"

void errorExit(const char* progname, const char* error) {
    if(error != NULL) {
        std::cerr << "ERROR: " << error << std::endl;
    }
    std::cerr << "usage: " << progname << " <matrix1> <matrix2>" << std::endl
              << "\twhere <matrix1> and <matrix2> are file names containing matrices." 
    		  << std::endl;
    exit( EXIT_FAILURE );
}

inline double secondsSince(  std::chrono::time_point<std::chrono::system_clock> alpha,
                             std::chrono::time_point<std::chrono::system_clock> omega ) {
	auto delta = std::chrono::duration_cast<std::chrono::microseconds>(omega - alpha);
	double result = delta.count( )  / 1000000.0;
	return result;
}

// +++ main starts here +++
int main(int argc, char** argv) {

    // process arguments
    if(argc != 3)
        errorExit(argv[0], "wrong number of arguments.");

    auto startTime = std::chrono::system_clock::now();

    CMatrix m1(argv[1]);                    // read 1st matrix
    CMatrix m2(argv[2]);                    // read 2nd matrix

    if (m1.width != m2.height)          // check compatibility
    	errorExit( argv[0], "Width of 1st matrix not equal to height of 2nd matrix.");

    auto milestoneSetup = std::chrono::system_clock::now();
    std::cerr << std::setprecision( 8 ) << "setup time = " 
              << secondsSince( startTime, milestoneSetup )
              << " seconds." << std::endl;

    // --- multiply matrices ---
    CMatrix result( m2.width, m1.height );      // allocate memory

    for(unsigned int row = 0; row < m1.height; row++) {
	    for(unsigned int col = 0; col < m2.width; col++) {
            double sum = 0.0;
            for(unsigned int dot = 0; dot < m2.height; dot++) {
                // cout <<  "m1[" << row << "][" << dot << "] = " << m1[row][dot]
                //  << ", m2[" << dot << "][" << col << "] = " << m2[dot][col] << endl;
                sum += m1[row][dot] * m2[dot][col];
            }
            // cout << "result[" << row <<" ][" << col << "] = " << sum;
            result[row][col] = sum;
        }
    }

    auto milestoneCalculate= std::chrono::system_clock::now();
    std::cerr << std::setprecision( 8 ) << "calculation time = "
              << secondsSince( milestoneSetup, milestoneCalculate )
              << " seconds." << std::endl;


    // print matrix
    result.print();

    auto endTime = std::chrono::system_clock::now();
    std::cerr << std::setprecision( 8 ) << "output time = " 
    		  << secondsSince( milestoneCalculate, endTime)
    		  << " seconds." << std::endl;
    std::cerr << std::setprecision( 8 ) << "Total wall clock time = " 
    		  << secondsSince( startTime, endTime)  
    		  << " seconds." << std::endl;

    return 0;
}

// EOF

