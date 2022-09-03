#pragma once

#include <fstream>
#include <sstream>
using namespace std;

vector<string> standings;
vector<tuple<int, int, int, int>> testResults;
string requestResult;

void apiUpdateStandings() {
    #ifdef _WIN32
      char cmd[] = "python ..\\api.py standings";
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
    int id, my, best, secondBest;
    while (getline(infile, s)) {
        stringstream ss(s);
        ss >> id >> my >> best >> secondBest;
        testResults.emplace_back(id, my, best, secondBest);
    }
    sort(testResults.begin(), testResults.end());
}

void apiSubmit(int task_id) {
    string sid = to_string(task_id);
    #ifdef _WIN32
      string cmd = "python ..\\api.py submit " + sid + " ..\\solutions\\" + sid + ".txt";
    #else
      string cmd = "python3 ../api.py submit " + sid + " ../solutions/" + sid + ".txt";
    #endif
    system(cmd.c_str());
    ifstream infile("req_result.txt");
    requestResult = "";
    string s;
    while (getline(infile, s)) {
        requestResult += s;
    }
}

void apiDownload(int task_id) {
    string sid = to_string(task_id);
    #ifdef _WIN32
      string cmd = "python ..\\api.py download " + sid + " ..\\solutions\\" + sid + ".txt";
    #else
      string cmd = "python3 ../api.py download " + sid + " ../solutions/" + sid + ".txt";
    #endif
    system(cmd.c_str());
    ifstream infile("req_result.txt");
    requestResult = "";
    string s;
    while (getline(infile, s)) {
        requestResult += s;
    }
}