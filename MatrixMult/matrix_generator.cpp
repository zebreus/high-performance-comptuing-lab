// file...: matrix_generator.cpp
// desc...: simple double matrix generator
// oct-2010 | a.knirsch@fbi.h-da.de

#include <assert.h>
#include <stdlib.h>

#include <iostream>
#include <random>
#include <sstream>

#include "CMatrix.h"

using namespace std;

void printError(const char* progname, const char* error) {
    if (error != NULL) {
        cerr << "ERROR: " << error << endl;
    }
    cerr << "usage: " << progname << " <width> <heigth>" << endl
         << "\t<width> and <heigth> are of type unsigned int" << endl;
}

bool convertToSize(const char* str, unsigned int& number) {
    bool success = false;
    if (str != NULL) {
        istringstream myStream(str);

        if (myStream >> number) {
            success = true;
        }
    }
    return success;
}

void createRandomDouble(double& number) {
    static std::random_device rd {};
    static std::mt19937 gen {rd()};
    static std::uniform_real_distribution<> d(-10000.0, 10000);
    number = d(gen);
}

// +++ main starts here +++
int main(int argc, char** argv) {
    // init
    double* container = NULL;
    unsigned int containersize = 0;
    unsigned int width = 0;
    unsigned int height = 0;

    // process arguments
    if (argc != 3) {
        printError(argv[0], "wrong number of arguments.");
        return -1;
    }

    if (!(convertToSize(argv[1], width) && convertToSize(argv[2], height))) {
        printError(argv[0], "failed to convert arguments to numbers.");
        return -2;
    }

    // create container
    containersize = width * height;
    container = new double[containersize];
    for (unsigned int i = 0; i < containersize; i++) {
        createRandomDouble(container[i]);
    }

    // print matrix
    cout << width << " " << height << endl;
    for (unsigned int i = 0; i < containersize; i++) {
        cout << fixed << container[i];
        if (((i + 1) % width) == 0) {
            cout << endl;
        } else {
            cout << " ";
        }
    }

    return 0;
}

// EOF
