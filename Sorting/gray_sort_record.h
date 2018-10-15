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
 
#pragma once

#include <istream>
#include <ostream>
#include <string.h> // just for memcmp() --- see "man memcmp"

 
class graySortRecord {
public:
	static const int record_size = 100;
	static const int key_size = 10;
	
private:
	char data[ record_size ];
	
public:
	// Constructors
	graySortRecord( ) = default; // simply copy the only member, data
	
	graySortRecord( const graySortRecord &other )  = default; // simple.
	
	// Destructor
	~graySortRecord() = default;
	
	// read only, "safe" getter for the data field
	// unsigned char *data( ) const { return data; }
	
	// overloaded "less than" operator
	bool operator <(const graySortRecord& rhs) const // rhs = right hand side
	{   return ( memcmp( data, rhs.data, key_size ) < 0 );  }

	// overloaded input
	friend std::istream& operator>>( std::istream& in, graySortRecord &rec )
	{	return in.read( rec.data, graySortRecord::record_size );  }
	
	// overloaded output
	friend std::ostream& operator<<( std::ostream& out, const graySortRecord& rec )
	{	return out.write( rec.data, graySortRecord::record_size );  }
	
};

