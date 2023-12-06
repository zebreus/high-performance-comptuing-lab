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

#include <string.h>  // just for memcmp() --- see "man memcmp"

#include <istream>
#include <ostream>

class GraySortRecord {
  public:
    static const int record_size = 100;
    static const int key_size = 10;

  public:
    char data[record_size];

  public:
    // Constructors
    GraySortRecord() = default;  // simply copy the only member, data

    GraySortRecord(const GraySortRecord& other) = default;  // simple.

    // Destructor
    ~GraySortRecord() = default;

    // read only, "safe" getter for the data field
    // unsigned char *data( ) const { return data; }

    // overloaded "less than" operator
    bool operator<(const GraySortRecord& rhs) const  // rhs = right hand side
    {
        return (memcmp(data, rhs.data, key_size) < 0);
    }

    // overloaded input
    friend std::istream& operator>>(std::istream& in, GraySortRecord& rec) {
        return in.read(rec.data, GraySortRecord::record_size);
    }

    // overloaded output
    friend std::ostream&
    operator<<(std::ostream& out, const GraySortRecord& rec) {
        return out.write(rec.data, GraySortRecord::record_size);
    }
};
