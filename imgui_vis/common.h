#pragma once

struct Log {
    stringstream s;

    Log() { s = stringstream(); }
    Log& clear() { s = stringstream(); return *this; }
    
    const char* c_str() const {
        return s.str().c_str();
    }
};

template <class T>
Log& operator<<(Log& l, const T& other) {
    l.s << other;
    return l;
}

Log msg;
int S = 10;
int optSeconds = 60;
bool optRunning;