#pragma once

struct Log {
    stringstream s;

    Log() { s = stringstream(); }
    Log& clear() { s = stringstream(); return *this; }
};

template <class T>
Log& operator<<(Log& l, const T& other) {
    l.s << other;
    return l;
}

Log msg;
int S = 10;
float T = 0.01;
int optSeconds = 60;
bool optRunning;
bool hardMove;