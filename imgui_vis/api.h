#pragma once

#include <fstream>
#include <sstream>
using namespace std;

vector<string> standings;
vector<tuple<int, int, int>> testResults;
string submitResult;

void apiUpdateStandings() {
    #ifdef _WIN32
      char cmd[] = "python3 ..\\api.py standings";
    #else
      char cmd[] = "python3 ../api.py standings";
    #endif
    system(cmd);

    ifstream infile("standings.txt");
    string s;
    standings.clear();
    while (getline(infile, s)) {
        standings.push_back(s);
    }

    infile = ifstream("tests.txt");
    testResults.clear();
    int id, my, best;
    while (getline(infile, s)) {
        stringstream ss(s);
        ss >> id >> my >> best;
        testResults.emplace_back(id, my, best);
    }
    sort(testResults.begin(), testResults.end());
}

void apiSubmit(int task_id) {
    string sid = to_string(task_id);
    #ifdef _WIN32
      string cmd = "python3 ..\\api.py submit " + sid + " ..\\solutions\\" + sid + ".txt";
    #else
      string cmd = "python3 ../api.py submit " + sid + " ../solutions/" + sid + ".txt";
    #endif
    system(cmd.c_str());
    ifstream infile("tests.txt");
    submitResult = "";
    string s;
    while (getline(infile, s)) {
        submitResult += s;
    }
}