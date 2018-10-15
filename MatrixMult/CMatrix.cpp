// file...: CMatrix.cpp
// desc...: matrix helper class implementation
// oct-2010 | a.knirsch@fbi.h-da.de
// June 2013: minor additions by <info@victor-hahn.de>
// Nov 2022: Minor additions from Herr Hahn finally merged

#include <iostream>
#include <fstream>
#include <assert.h>
#include "CMatrix.h"

// using namespace std;

// create matrix from given file (input!)
CMatrix::CMatrix(const char* filename) {
    width = 0;
    height = 0;
    FILE* pFile = NULL;
    double value = 0.0;
    unsigned int i = 0;

    pFile = fopen (filename,"r");
    if (nullptr == pFile) {
        perror ("Error opening file");
    } else {
        // Note: fscanf returns "the number of input items assigned" (according to man)
        int status = fscanf(pFile, "%u", &width);
        assert (1 == status);
        status = fscanf(pFile, "%u", &height);
        assert(1 == status);
        
        container = new double[size()];
        while(EOF != fscanf(pFile, "%lf", &value)) {
            container[i++] = value;
        }
        fclose(pFile);
        assert(size() == i);
    }
}

// create empty matrix with given size
CMatrix::CMatrix(unsigned int w, unsigned int h) {
    width = w;
    height = h;
    container = new double[size()];
    for(unsigned int i=0; i<size(); i++) {
        container[i] = 0.0;
    }
}

// Copy Constructors
CMatrix::CMatrix(const CMatrix& rhs)  // rhs == right hand side
{
   width = rhs.width;
   height = rhs.height;
   container = new double[size()];
   for (unsigned int i = 0; i < size(); i++)
       container[i] = rhs.container[i];
}

CMatrix& CMatrix::operator=( const CMatrix& rhs )
{
	// if rhs has different dimensions than we do, trash our data.
   if (( rhs.width != width) && ( rhs.height != height)) 
   {
   		delete[] container;
	   	width     = rhs.width;
   		height    = rhs.height;
   		container = new double[ size() ];
   };
   for (unsigned int i = 0; i < size(); i++)
       container[i] = rhs.container[i];
   return *this;
}

// Destructor
CMatrix::~CMatrix() { delete[] container; container = NULL; }

// Output 
void CMatrix::print() const {
    std::cout << width << " " << height << std::endl;
    for(unsigned int i=0; i<size(); i++) {
        std::cout << container[i];
        if( 0 == ((i+1)%width) )
            std::cout << std::endl;
        else
            std::cout << " ";
    } // end for all numbers in matrix
}
