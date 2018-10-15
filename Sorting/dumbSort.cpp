/* -*- Mode: C++; c-basic-offset:4 ; -*- */
/*  This file is meant to demonstrate a simple (naive) as a simple
 *  C++ wrapper for the records sorted in the "Gray" sorting benchmark.
 * 
 *  The code is original, written by Prof. Dr. Ronald Moore, h_da,
 *  ronald.moore@h-da.de, but the ideas are all from 
 *  http://sortbenchmark.org/    and
 *  http://www.ordinal.com/gensort.html
 * 
 *  This code is provided with no license, is totally open,
 *  but comes with no warranty or guarantee for any purpose whatsoever.
 */

#include <iostream>
#include <iomanip>
#include <fstream>
#include <vector>
#include <algorithm>
#include <chrono>

#include "gray_sort_record.h"

double secondsSince( std::chrono::system_clock::time_point startTime ) 
{
	const double microsecs = 1000000.0;
    auto endTime = std::chrono::system_clock::now();
    auto microRunTime = std::chrono::duration_cast<
                                std::chrono::microseconds>(endTime - 
                                                           startTime);
    return microRunTime.count() / microsecs;
};	

void printTime( std::chrono::system_clock::time_point startTime ) 
{
	std::cout << "Wall Time used so far: " 
	          << std::setprecision( 8 )
	          << secondsSince( startTime ) 
	          << " seconds." << std::endl;
};

int main ( int argc, char **argv ) 
{
	std::cout << "Dumb Sort Program starts..." << std::endl;
	
	// check command line usage
	if ( 3 != argc ) {
		std::cerr << "Usage: " << argv[0] 
		   << " <input binary data file name> <output binary data file name>\n"
		   << "Example " << argv[0] << " foo.dat bar.dat\n";
		exit( -1 );
	}; // else... 
	
	// start stop watch...
	auto startTime = std::chrono::system_clock::now();

	// open input file for binary input
	std::ifstream inStream( argv[ 1 ],  std::ios::in|std::ios::binary );
	if ( ! inStream.is_open()) {
		std::cerr << "ERROR: Could not open input file named " 
		          << argv[ 1 ] << std::endl;
		exit( -2 );
	}; // else... 
	
	// open output file for binary output
	std::ofstream outStream( argv[ 2 ], std::ios::out|std::ios::binary );
	if ( ! outStream.is_open()) {
		std::cerr << "ERROR: Could not open output file named " 
		          << argv[ 2 ] << std::endl;
		exit( -2 );
	}; // else... 
	
	// read all of the records into a vector
	std::vector< graySortRecord > records;
	records.reserve( 1024 ); // to save copying, start with 1 KByte data 
	graySortRecord rec;
	long int numRecords = 0;
	while ( inStream >> rec ) {
		records.push_back( rec );
		++numRecords;
	};
	std::cout << "Read " << numRecords << " from input file!\n";
	printTime( startTime );
	
	// SORT THE RECORDS!!!
	std::sort( records.begin(), records.end() );
	
	std::cout << "Finished Sorting!\n";
	printTime( startTime );

	// write all of the records to output file
	for ( graySortRecord r : records ) 
		outStream << r;
	
	// Stop stopwatch!
	std::cout << "Dumb Sort Program finished!" << std::endl;
	printTime( startTime );

	return ( 0 ); // OK!!!	                 
}
