
#include "imgui.h"
#include "imgui_impl_sdl.h"
#include "imgui_impl_opengl3.h"
#include <stdio.h>
#include <SDL.h>
#if defined(IMGUI_IMPL_OPENGL_ES2)
#include <SDL_opengles2.h>
#else
#include <SDL_opengl.h>
#endif

#include "sdl_system.h"

#include <random>
#include <functional>
#include <filesystem>
#include <iostream>
#include <cmath>
#include <vector>
#include <unordered_map>
#include <algorithm>
#include <array>
#include <thread>
#include <cassert>
#include <tuple>
#include <chrono>
#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#endif

#include "api.h"

using namespace std;
namespace fs = std::filesystem;

typedef std::chrono::high_resolution_clock Time;
typedef std::chrono::milliseconds chrono_ms;
    
#define forn(i, N) for (int i = 0; i < (int)(N); i++)
#define sqr(x) (x)*(x)
using ll = long long;
using Color = array<int, 4>;

constexpr int tColor = 1;
constexpr int tSplitPoint = 2;
constexpr int tSplitX = 3;
constexpr int tSplitY = 4;
constexpr int tMerge = 5;
constexpr int tSwap = 6;

const string inputsPath = "../inputs/";
const string solutionsPath = "../solutions/";


int N, M;
vector<vector<Color>> colors;
bool running;

struct Instruction {
    string id, oid;
    int type;
    int x, y;
    Color color;

    string text() const {
        char buf[128];
        if (type == tColor) {
            sprintf(buf, "color [%s] [%d, %d, %d, %d]", id.c_str(), color[0], color[1], color[2], color[3]);
        } else if (type == tSplitPoint) {
            sprintf(buf, "cut [%s] [%d, %d]", id.c_str(), x, N - y);
        } else if (type == tSplitX) {
            sprintf(buf, "cut [%s] [X] [%d]", id.c_str(), x);
        } else if (type == tSplitY) {
            sprintf(buf, "cut [%s] [Y] [%d]", id.c_str(), N - y);
        } else if (type == tMerge) {
            sprintf(buf, "merge [%s] [%s]", id.c_str(), oid.c_str());
        } else if (type == tSwap) {
            sprintf(buf, "swap [%s] [%s]", id.c_str(), oid.c_str());
        } else assert(false);
        return buf;
    }
};

Instruction ColorIns(const string& i, Color c) {
    Instruction res;
    res.type = tColor;
    res.id = i;
    res.color = c;
    return res;
}

Instruction SplitPointIns(const string& i, int x, int y) {
    Instruction res;
    res.type = tSplitPoint;
    res.id = i;
    res.x = x;
    res.y = y;
    return res;
}

Instruction SplitXIns(const string& i, int x) {
    Instruction res;
    res.type = tSplitX;
    res.id = i;
    res.x = x;
    return res;
}

Instruction SplitYIns(const string& i, int y) {
    Instruction res;
    res.type = tSplitY;
    res.id = i;
    res.y = y;
    return res;
}

Instruction MergeIns(const string& i1, string i2) {
    Instruction res;
    res.type = tMerge;
    res.id = i1;
    res.oid = i2;
    return res;
}

Instruction SwapIns(const string& i1, const string& i2) {
    Instruction res;
    res.type = tSwap;
    res.id = i1;
    res.oid = i2;
    return res;
}

struct Solution {
    double score;
    vector<Instruction> ins;
};

struct Block {
    int r1, c1, r2, c2;
};

struct Painter {
    int lastBlockId;
    int N, M;
    unordered_map<string, Block> blocks;
    vector<vector<Color>> clr;
    double opsScore;

    Painter() {}
    Painter(int n, int m) {
        lastBlockId = 0;
        opsScore = 0;
        N = n;
        M = m;
        Color c;
        c[0] = c[1] = c[2] = c[3] = 255;
        clr.assign(n, vector<Color>(m, c));
        for (int i = 0; i < n; i++)
            for (int j = 0; j < m; j++)
                clr[i][j] = c;
        blocks["0"] = Block{0, 0, N, M};
    }

    bool doColor(const string& i, Color c) {
        if (blocks.find(i) == blocks.end())
            return false;
        const auto& b = blocks[i];
        opsScore += round(5.0 * N * M / ((b.r2 - b.r1) * (b.c2 - b.c1)));
        for (int i = b.r1; i < b.r2; i++)
            for (int j = b.c1; j < b.c2; j++)
                clr[i][j] = c;
        return true;
    }

    bool doSplitX(const string& i, int x) {
        if (blocks.find(i) == blocks.end())
            return false;
        const auto& b = blocks[i];
        if (x <= b.c1 || x >= b.c2) return false;
        opsScore += round(7.0 * N * M / ((b.r2 - b.r1) * (b.c2 - b.c1)));
        Block left = b;
        Block right = b;
        blocks.erase(blocks.find(i));
        left.c2 = x;
        right.c1 = x;
        blocks[i + ".0"] = left;
        blocks[i + ".1"] = right;
        return true;
    }

    bool doSplitY(const string& i, int y) {
        if (blocks.find(i) == blocks.end())
            return false;
        const auto& b = blocks[i];
        if (y <= b.r1 || y >= b.r2) return false;
        opsScore += round(7.0 * N * M / ((b.r2 - b.r1) * (b.c2 - b.c1)));
        Block down = b;
        Block up = b;
        blocks.erase(blocks.find(i));
        down.r1 = y;
        up.r2 = y;
        blocks[i + ".0"] = down;
        blocks[i + ".1"] = up;
        return true;
    }

    bool doSplitPoint(const string& i, int x, int y) {
        if (blocks.find(i) == blocks.end())
            return false;
        const auto& b = blocks[i];
        if (x <= b.c1 || x >= b.c2) return false;
        if (y <= b.r1 || y >= b.r2) return false;
        opsScore += round(10.0 * N * M / ((b.r2 - b.r1) * (b.c2 - b.c1)));
        Block b0 = b;
        Block b1 = b;
        Block b2 = b;
        Block b3 = b;
        blocks.erase(blocks.find(i));
        b3.r2 = y; b2.r2 = y;
        b0.r1 = y; b1.r1 = y;
        b3.c2 = x; b0.c2 = x;
        b1.c1 = x; b2.c1 = x;
        blocks[i + ".0"] = b0;
        blocks[i + ".1"] = b1;
        blocks[i + ".2"] = b2;
        blocks[i + ".3"] = b3;
        return true;
    }

    bool doMerge(const string& i1, const string& i2) {
        if (blocks.find(i1) == blocks.end() || blocks.find(i2) == blocks.end())
            return false;

        const auto& bu = blocks[i1];
        const auto& bv = blocks[i2];
        opsScore += round(1.0 * N * M / max((bu.r2 - bu.r1) * (bu.c2 - bu.c1),
                                            (bv.r2 - bv.r1) * (bv.c2 - bv.c1)));
        Block nb;
        if (bu.r2 == bv.r1 || bu.r1 == bv.r2) {
            if (bu.c1 == bv.c1 && bu.c2 == bv.c2) {
                if (bu.r2 == bv.r1) {
                    nb = bu;
                    nb.r2 = bv.r2;
                } else {
                    nb = bv;
                    nb.r2 = bu.r2;
                }
            } else {
                return false;
            }
        }

        if (bu.c2 == bv.c1 || bu.c1 == bv.c2) {
            if (bu.r1 == bv.r1 && bu.r2 == bv.r2) {
                if (bu.c2 == bv.c1) {
                    nb = bu;
                    nb.c2 = bv.c2;
                } else {
                    nb = bv;
                    nb.c2 = bu.c2;
                }
            } else {
                return false;
            }
        }

        blocks.erase(blocks.find(i1));
        blocks.erase(blocks.find(i2));
        lastBlockId++;
        blocks[to_string(lastBlockId)] = nb;
        return true;
    }

    bool doSwap(const string& i1, const string& i2) {
        return false;
        if (blocks.find(i1) == blocks.end() || blocks.find(i2) == blocks.end())
            return false;

        const auto& bu = blocks[i1];
        const auto& bv = blocks[i2];
        opsScore += round(1.0 * N * M / max((bu.r2 - bu.r1) * (bu.c2 - bu.c1),
                                            (bv.r2 - bv.r1) * (bv.c2 - bv.c1)));
        Block nb;
        if (bu.r2 == bv.r1 || bu.r1 == bv.r2) {
            if (bu.c1 == bv.c1 && bu.c2 == bv.c2) {
                if (bu.r2 == bv.r1) {
                    nb = bu;
                    nb.r2 = bv.r2;
                } else {
                    nb = bv;
                    nb.r2 = bu.r2;
                }
            } else {
                return false;
            }
        }

        if (bu.c2 == bv.c1 || bu.c1 == bv.c2) {
            if (bu.r1 == bv.r1 && bu.r2 == bv.r2) {
                if (bu.c2 == bv.c1) {
                    nb = bu;
                    nb.c2 = bv.c2;
                } else {
                    nb = bv;
                    nb.c2 = bu.c2;
                }
            } else {
                return false;
            }
        }

        blocks.erase(blocks.find(i1));
        blocks.erase(blocks.find(i2));
        lastBlockId++;
        blocks[to_string(lastBlockId)] = nb;
        return true;
    }

    bool doInstruction(const Instruction& ins) {
        if (ins.type == tColor) {
            return doColor(ins.id, ins.color);            
        } else if (ins.type == tSplitPoint) {
            return doSplitPoint(ins.id, ins.x, ins.y);
        } else if (ins.type == tSplitX) {
            return doSplitX(ins.id, ins.x);
        } else if (ins.type == tSplitY) {
            return doSplitY(ins.id, ins.y);
        } else if (ins.type == tMerge) {
            return doMerge(ins.id, ins.oid);
        } else if (ins.type == tSwap) {
            return doSwap(ins.id, ins.oid);
        } else return false;
    }

    int totalScore(const vector<vector<Color>>& targetColors) const {
        double res = 0;
        // cerr << "opsScore = " << opsScore << endl;
        // cerr << clr.size() << " " << targetColors.size() << endl;
        for (int i = 0; i < N; i++)
            for (int j = 0; j < M; j++) {
                double d = 0;
                for (int q = 0; q < 4; q++)
                    d += sqr(clr[i][j][q] - targetColors[i][j][q]);
                res += sqrt(d);
            }
        return round(res * 0.005 + opsScore);
    }
};

struct Input {
    int N, M;
    vector<vector<Color>> colors;
};

int selected_idx, currentTestId;
unordered_map<ll, Solution> mem;
int S = 10;
Painter painter;
bool useSplitX, useSplitPoint, useSplitY;
unordered_map<int, int> myScores;
string msg;

double scale = 1;
double shiftX, shiftY;


Input readInput(const string& fname) {
    freopen(fname.c_str(), "r", stdin);
    Input res;
    cin >> res.N >> res.M;
    res.colors.assign(res.N, vector<Color>(res.M, Color()));
    for (int i = 0; i < res.N; i++)
        for (int j = 0; j < res.M; j++)
            for (int q = 0; q < 4; q++)
                cin >> res.colors[i][j][q];
    return res;
}

void readInputAndStoreAsGlobal(const string& fname) {
    Input i = readInput(fname);
    N = i.N;
    M = i.M;
    colors = i.colors;
}

void postprocess(const Solution& res) {
    painter = Painter(N, M);
    msg += "Solved with penalty " + to_string(res.score) + "\n";
    for (const auto& ins : res.ins) {
        if (!painter.doInstruction(ins)) {
            msg += "Bad instruction: " + ins.text() + "\n";
            return;
        }
    }
    msg += "Painter score: " + to_string(painter.totalScore(colors)) + "\n";
    if (myScores[currentTestId] == -1 || res.score < myScores[currentTestId]) {
        string fname = "../solutions/" + to_string(currentTestId) + ".txt";
        myScores[currentTestId] = res.score;
        ofstream ofs(fname);
        for (const auto& i : res.ins) {
            ofs << i.text() << endl;
        }
        ofs.close();

        ofs = ofstream("local_scores.txt");
        for (auto [id, sc] : myScores)
            ofs << id << " " << sc << endl;
        ofs.close();
    }
}

Solution loadSolution(const Input& in, const string& filepath) {
    Solution res;
    res.score = -1;
    ifstream infile(filepath);
    string s, token, id, oid;
    int val;
    while (getline(infile, s)) {
        string cs = "";
        for (auto c : s)
            if (c != '[' && c != ']') {
                if (c == ',') cs += ' ';
                else cs += c;
            }

        if (cs.substr(0, 3) == "cut") {
            stringstream ss(cs.substr(4));
            ss >> id;
            ss >> token;
            ss >> val;
            // cerr << cs << ": " << id << " " << token << " " << val << "(" << in.N << " " << in.M << ")" << endl;
            if (token == "X") {
                res.ins.push_back(SplitXIns(id, val));
            } else if (token == "Y") {
                res.ins.push_back(SplitYIns(id, in.N - val));
            } else {
                res.ins.push_back(SplitPointIns(id, stoi(token), in.N - val));
            }
        } else if (cs.substr(0, 5) == "merge") {
            stringstream ss(cs.substr(6));
            ss >> id >> oid;
            res.ins.push_back(MergeIns(id, oid));
        } else if (cs.substr(0, 4) == "swap") {
            stringstream ss(cs.substr(5));
            ss >> id >> oid;
            res.ins.push_back(SwapIns(id, oid));
        } else if (cs.substr(0, 5) == "color") {
            stringstream ss(cs.substr(6));
            Color c;
            ss >> id >> c[0] >> c[1] >> c[2] >> c[3];
            res.ins.push_back(ColorIns(id, c));
        } else {
            cerr << "Unsupported instruction: " << cs << " in file " << filepath << "\n";
            return res;
        }
    }
    Painter p(in.N, in.M);
    for (const auto& ins : res.ins) {
        if (!p.doInstruction(ins)) {
            cerr << "Bad instruction in " + s + ": " + ins.text() + "\n";
            res.score = -100;
            return res;
        }
    }
    res.score = p.totalScore(in.colors);
    return res;
}

void updateStandingsAndMyScores(bool useApiUpdate) {
    if (useApiUpdate) apiUpdateStandings();
    /*std::ofstream ofs("my_scores.txt");
    string solutionsPath = "../solutions/";
    string inputsPath = "../inputs/";
    for (const auto & entry : fs::directory_iterator(solutionsPath)) {
        string s = entry.path().string();

        size_t i = 0;
        while (i < s.size() && (s[i] < '0' || s[i] > '9')) i++;
        if (i >= s.size()) continue;
        size_t j = i;
        while (s[j] >= '0' && s[j] <= '9') j++;

        int test_id;
        sscanf(s.substr(i, j).c_str(), "%d", &test_id);

        Input in = readInput(inputsPath + to_string(test_id) + ".txt");
        cerr << "read input " << in.N << "x" << in.M << endl;
        Solution sol = loadSolution(in, s);
        Painter p(in.N, in.M);
        for (const auto& ins : sol.ins) {
            if (!p.doInstruction(ins)) {
                cerr << "Bad instruction in " + s + ": " + ins.text() + "\n";
                sol.score = -100;
                break;
            }
        }
        if (sol.score > -99) sol.score = p.totalScore(in.colors);
        cerr << test_id << " " << sol.score << endl;
        myScores[test_id] = round(sol.score);
    }
    ofs.close();*/
    std::ifstream ifs("local_scores.txt");
    int test_id, score;
    while (ifs >> test_id >> score) {
        myScores[test_id] = score;
    }
}

chrono::high_resolution_clock::time_point lastUpdateTime;

void updateStandingsTimed() {
    /*
    auto curTime = chrono::high_resolution_clock::now();

    if (chrono::duration_cast<chrono::seconds>(curTime - lastUpdateTime).count() > 30) {
        updateStandingsAndMyScores();
        lastUpdateTime = curTime;
    }*/
    for (int i = 0; running; i++) {
        #ifdef _WIN32
            if (i % 30 == 0)
                updateStandingsAndMyScores(false);
            Sleep(1000);
        #else
            if (i % 30 == 0)
                updateStandingsAndMyScores(true);
            sleep(1);
        #endif
    }
}

void downloadSolution(int testId) {
    currentTestId = testId;
    apiDownload(testId);
    Input in = readInput(inputsPath + to_string(currentTestId) + ".txt");
    N = in.N;
    M = in.M;
    colors = in.colors;
    Solution sol = loadSolution(in, solutionsPath + to_string(currentTestId) + ".txt");
    postprocess(sol);
    cerr << "downloaded and loaded sol with score " << sol.score << endl;
}

void fileWindow() {
    if(ImGui::Begin("Tests")) {
        if (ImGui::Button("Update")) {
            updateStandingsAndMyScores(true);
        }

        ImGui::SameLine(70);
        if (ImGui::Button("Download Better")) {
            for (const auto & entry : fs::directory_iterator(inputsPath)) {
                string s = entry.path().string();
                size_t i = 0;
                while (i < s.size() && (s[i] < '0' || s[i] > '9')) i++;
                if (i >= s.size()) continue;
                size_t j = i;
                while (s[j] >= '0' && s[j] <= '9') j++;
                int idx;
                sscanf(s.substr(i, j).c_str(), "%d", &idx);
                if (idx - 1 < (int)testResults.size()) {
                    if (get<1>(testResults[idx - 1]) < myScores[idx] || myScores[idx] == -1) {
                        downloadSolution(idx);
                    }    
                }
            }
        }

        ImGui::SameLine(200);
        if (ImGui::Button("Read Local")) {
            for (const auto & entry : fs::directory_iterator(inputsPath)) {
                string s = entry.path().string();
                size_t i = 0;
                while (i < s.size() && (s[i] < '0' || s[i] > '9')) i++;
                if (i >= s.size()) continue;
                size_t j = i;
                while (s[j] >= '0' && s[j] <= '9') j++;
                int test_id;
                sscanf(s.substr(i, j).c_str(), "%d", &test_id);
                
                Input in = readInput(s);
                cerr << "read input " << in.N << "x" << in.M << endl;
                Solution sol = loadSolution(in, solutionsPath + to_string(test_id) + ".txt");
                Painter p(in.N, in.M);
                for (const auto& ins : sol.ins) {
                    if (!p.doInstruction(ins)) {
                        cerr << "Bad instruction in " + s + ": " + ins.text() + "\n";
                        sol.score = -100;
                        break;
                    }
                }
                if (sol.score > -99) sol.score = p.totalScore(in.colors);
                cerr << test_id << " " << sol.score << endl;
                myScores[test_id] = round(sol.score);
            }

            ofstream ofs("local_scores.txt");
            for (auto [id, sc] : myScores)
                ofs << id << " " << sc << endl;
            ofs.close();
        }

        vector<pair<int, string>> tests;
        for (const auto & entry : fs::directory_iterator(inputsPath)) {
            string s = entry.path().string();
            tests.emplace_back(0, s);
            size_t i = 0;
            while (i < s.size() && (s[i] < '0' || s[i] > '9')) i++;
            if (i >= s.size()) continue;
            size_t j = i;
            while (s[j] >= '0' && s[j] <= '9') j++;
            sscanf(s.substr(i, j).c_str(), "%d", &tests.back().first);
        }

        sort(tests.begin(), tests.end());
        if (ImGui::BeginTable("Tests", 5))
        {
            ImGui::TableSetupColumn("ID");
            ImGui::TableSetupColumn("Local");
            ImGui::TableSetupColumn("My subs");
            ImGui::TableSetupColumn("Best");
            ImGui::TableSetupColumn("Loss");
            ImGui::TableHeadersRow();

            for (size_t idx = 0; idx < tests.size(); idx++) {
                ImGui::TableNextRow();
                ImGui::TableNextColumn();
                int tid = tests[idx].first;
                string bName = "Load " + to_string(tid);
                if (ImGui::Button(bName.c_str())) {
                    currentTestId = tests[idx].first;
                    readInputAndStoreAsGlobal(tests[idx].second);
                    cerr << "load " << tid << " " << N << " " << M << endl;
                    requestResult = "";
                }
                ImGui::TableNextColumn();
                ImGui::Text("%d", myScores[tests[idx].first]);                
                ImGui::SameLine(55);
                bName = "Submit " + to_string(tid);
                if (ImGui::Button(bName.c_str())) {
                    apiSubmit(tests[idx].first);
                }
                
                if (idx < testResults.size()) {
                    assert(get<0>(testResults[idx]) == tests[idx].first);
                    ImGui::TableNextColumn();
                    ImGui::Text("%d", get<1>(testResults[idx]));
                    ImGui::SameLine(55);
                    bName = "Download " + to_string(tid);
                    if (ImGui::Button(bName.c_str())) {
                        downloadSolution(tests[idx].first);
                    }

                    ImGui::TableNextColumn();
                    ImGui::Text("%d", get<2>(testResults[idx]));
                    ImGui::TableNextColumn();
                    ImGui::Text("%d", get<1>(testResults[idx]) - get<2>(testResults[idx]));
                } else {
                    ImGui::TableNextColumn();
                    ImGui::Text("N/A");
                    ImGui::TableNextColumn();
                    ImGui::Text("N/A");
                    ImGui::TableNextColumn();
                    ImGui::Text("N/A");
                }
            }
            ImGui::EndTable();
        }
    }

    ImGui::End();
}

void draw() {
    ImDrawList* dl = ImGui::GetBackgroundDrawList();
    auto QP = [](double x, double y) {
        return ImVec2(x * scale - shiftX, y * scale - shiftY);
    };
    for (int i = 0; i < N; i++)
        for (int j = 0; j < M; j++) {
            ImU32 color = IM_COL32(colors[i][j][0], colors[i][j][1], colors[i][j][2], colors[i][j][3]);
            dl->AddRectFilled(QP(j, i), QP((j + 1), (i + 1)), color);

            if (i < (int)painter.clr.size() && j < (int)painter.clr[i].size()) {
                color = IM_COL32(painter.clr[i][j][0], painter.clr[i][j][1],
                                 painter.clr[i][j][2], painter.clr[i][j][3]);
                dl->AddRectFilled(QP(j + M + 10, i), QP((j + 1 + M + 10), (i + 1)), color);
            }
        }
}


void processMouse() {
    auto& io = ImGui::GetIO();
    if (io.WantCaptureMouse) return;
    if (io.MouseWheel == 1) {
        scale = scale * 1.1;
    }
    if (io.MouseWheel == -1) {
        scale = scale / 1.1;
    }
    if (ImGui::IsMouseDown(1)) {
        shiftX -= io.MouseDelta.x;
        shiftY -= io.MouseDelta.y;
    }
    if (ImGui::IsMouseDown(0)) {
    }
    if (ImGui::IsMouseReleased(0)) {
    }
}

int totalNodes;
int visitedNodes;

Solution getInstructions(string id, int r1, int c1, int r2, int c2) {
    ll key = ((r1 * M + c1) * ll(N) + r2) * ll(M) + c2;
    if (mem.find(key) != mem.end()) {
        auto res = mem[key];
        for (size_t w = 0; w < res.ins.size(); w++)
            res.ins[w].id = id + res.ins[w].id;
        return res;
    }
    // cerr << r1 << "," << c1 << " " << r2 << "," << c2 << endl;
    visitedNodes++;
    if (visitedNodes % 100 == 0) {
        msg = "Processed " + to_string(visitedNodes) + " of " + to_string(totalNodes) + "...";
    }

    Color sum;
    for (int q = 0; q < 4; q++) sum[q] = 0;
    int total = 0;
    for (int r = r1; r < r2; r++)
        for (int c = c1; c < c2; c++) {
            for (int q = 0; q < 4; q++)
                sum[q] += colors[r][c][q];
            total++;
        }

    for (int q = 0; q < 4; q++)
        sum[q] /= total;

    double penalty = round(5.0 * N * M / ((r2 - r1) * (c2 - c1)));
    double colorPenalty = 0.0;
    for (int r = r1; r < r2; r++)
        for (int c = c1; c < c2; c++) {
            double ssq = 0;
            for (int q = 0; q < 4; q++)
                ssq += sqr(sum[q] - colors[r][c][q]);
            colorPenalty += sqrt(ssq);
        }
    penalty += colorPenalty * 0.005;

    Solution res;
    res.ins.push_back(ColorIns("", sum));
    res.score = penalty;

    double cscore = 7.0 * N * M / ((r2 - r1) * (c2 - c1));
    if (useSplitY) {
        for (int y = r1 + S; y < r2; y += S) {
            Solution s1 = getInstructions(".1", r1, c1, y, c2);
            Solution s2 = getInstructions(".0", y, c1, r2, c2);
            if (cscore + s1.score + s2.score < res.score) {
                res.score = cscore + s1.score + s2.score;
                res.ins.clear();
                res.ins.push_back(SplitYIns("", y));
                res.ins.insert(res.ins.end(), s1.ins.begin(), s1.ins.end());
                res.ins.insert(res.ins.end(), s2.ins.begin(), s2.ins.end());
            }
        }
    }

    if (useSplitX) {
        for (int x = c1 + S; x < c2; x += S) {
            Solution s1 = getInstructions(".0", r1, c1, r2, x);
            Solution s2 = getInstructions(".1", r1, x, r2, c2);
            if (cscore + s1.score + s2.score < res.score) {
                res.score = cscore + s1.score + s2.score;
                res.ins.clear();
                res.ins.push_back(SplitXIns("", x));
                res.ins.insert(res.ins.end(), s1.ins.begin(), s1.ins.end());
                res.ins.insert(res.ins.end(), s2.ins.begin(), s2.ins.end());
            }
        }
    }

    if (useSplitPoint) {
        cscore = 10.0 * N * M / ((r2 - r1) * (c2 - c1));
        for (int x = c1 + S; x < c2; x += S)
            for (int y = r1 + S; y < r2; y += S) {
                Solution s0 = getInstructions(".0", y, c1, r2, x);
                Solution s1 = getInstructions(".1", y, x, r2, c2);
                Solution s2 = getInstructions(".2", r1, x, y, c2);
                Solution s3 = getInstructions(".3", r1, c1, y, x);
                if (cscore + s1.score + s2.score + s3.score + s0.score < res.score) {
                    res.score = cscore + s1.score + s2.score + s3.score + s0.score;
                    res.ins.clear();
                    res.ins.push_back(SplitPointIns("", x, y));
                    res.ins.insert(res.ins.end(), s0.ins.begin(), s0.ins.end());
                    res.ins.insert(res.ins.end(), s3.ins.begin(), s3.ins.end());
                    res.ins.insert(res.ins.end(), s1.ins.begin(), s1.ins.end());
                    res.ins.insert(res.ins.end(), s2.ins.begin(), s2.ins.end());
                }
            }
    }

    // cerr << r1 << "," << c1 << " " << r2 << "," << c2 << " - ";
    // for (const auto& i : res.ins) cerr << " " << i.text();
    // cerr << endl;
    mem[key] = res;
    for (size_t w = 0; w < res.ins.size(); w++)
        res.ins[w].id = id + res.ins[w].id;
    return res;
}

void solveDP() {
    mem.clear();
    msg = "Running...";
    totalNodes = (N / S + 2) * (N / S + 1) * (M / S + 2) * (M / S + 1) / 4;
    visitedNodes = 0;
    Solution res = getInstructions("0", 0, 0, N, M);
    msg = "";
    postprocess(res);
}

int dp[20100][20100];
int aux[201][201];

int mode = 0;

void solveGena() {
    if (S < 2) {
      msg = "sorry, S must be at least 2";
      return;
    }
    if (N % S != 0 || M % S != 0) {
      msg = "sorry, N and M should be divisible by S";
      return;
    }
    auto start_time = Time::now();
    auto GetTime = [&]() {
      auto cur_time = Time::now();
      std::chrono::duration<double> fs = cur_time - start_time;
      return std::chrono::duration_cast<chrono_ms>(fs).count() * 0.001;
    };
    msg = "Running...";
    int n = N / S;
    int m = M / S;
    vector<vector<vector<int>>> pref(N + 1, vector<vector<int>>(M + 1, vector<int>(4)));
    for (int i = 0; i <= N; i++) {
      for (int j = 0; j <= M; j++) {
        for (int k = 0; k < 4; k++) {
          if (i == 0 || j == 0) {
            pref[i][j][k] = 0;
          } else {
            pref[i][j][k] = pref[i - 1][j][k] + pref[i][j - 1][k] - pref[i - 1][j - 1][k] + colors[i - 1][j - 1][k];
          }
        }
      }
    }
    const int MAX_D = 255 * 255 * 4;
    vector<double> SQRT(MAX_D + 1);
    for (int i = 0; i <= MAX_D; i++) {
      SQRT[i] = sqrt(i);
    }
    auto PaintCost = [&](int x, int y) -> int {
      assert(x > 0 && y > 0);
      if (x == n && y == m) {
        return 5;
      }
      if (x == n) {
        return 7 + llround(5.0 * m / y) + llround(1.0 * m / max(y, m - y));
      }
      if (y == m) {
        return 7 + llround(5.0 * n / x) + llround(1.0 * n / max(x, n - x));
      }
      int ret = 10 + llround(5.0 * (n * m) / (x * y));
      int cand1 = llround(1.0 * (n * m) / (max(x, n - x) * y));
      cand1    += llround(1.0 * (n * m) / (max(x, n - x) * (m - y)));
      cand1    += llround(1.0 * m / max(y, m - y));
      int cand2 = llround(1.0 * (n * m) / (x * max(y, m - y)));
      cand2    += llround(1.0 * (n * m) / ((n - x) * max(y, m - y)));
      cand2    += llround(1.0 * n / max(x, m - x));
      return ret + min(cand1, cand2);
    };
    {
      assert(N == M);
      int tmp = 0;
      for (int xa = 0; xa < n; xa++) {
        for (int xb = xa + 1; xb <= n; xb++) {
          aux[xa][xb] = tmp++;
        }
      }
    }
    mt19937 rng(58);
    for (int xa = n - 1; xa >= 0; xa--) {
      auto time_elapsed = GetTime();
      msg = "n = " + to_string(n) + ", xa = " + to_string(xa) + ", time = " + to_string(time_elapsed) + " s";
      for (int ya = m - 1; ya >= 0; ya--) {
        for (int xb = xa + 1; xb <= n; xb++) {
          for (int yb = ya + 1; yb <= m; yb++) {
            int ft = (int) 1e9;
            for (int x = xa + 1; x < xb; x++) {
              ft = min(ft, dp[aux[xa][x]][aux[ya][yb]] + dp[aux[x][xb]][aux[ya][yb]]);
            }
            for (int y = ya + 1; y < yb; y++) {
              ft = min(ft, dp[aux[xa][xb]][aux[ya][y]] + dp[aux[xa][xb]][aux[y][yb]]);
            }
            int area = (xb - xa) * (yb - ya) * S * S;
            Color paint_into;
            for (int k = 0; k < 4; k++) {
              int sum = pref[yb * S][xb * S][k] - pref[ya * S][xb * S][k] - pref[yb * S][xa * S][k] + pref[ya * S][xa * S][k];
              paint_into[k] = (2 * sum + area) / (2 * area);
            }
            long long penalty = 1000 * PaintCost((mode & 1) ? xb : n - xa, (mode & 2) ? yb : m - ya);
            if (penalty < ft) {
              double diff_est = 0;
              if (area >= S) {
                for (int y = ya * S; y < yb * S; y++) {
                  int x = xa * S + (int) (rng() % (xb * S - xa * S));
                  int sum_sq = 0;
                  for (int k = 0; k < 4; k++) {
                    sum_sq += sqr(colors[y][x][k] - paint_into[k]);
                  }
                  diff_est += SQRT[sum_sq];
                }
              }
              diff_est *= xb * S - xa * S;
              if (penalty + llround(diff_est * 5 * 0.8) < ft) {
                double diff = 0;
                for (int y = ya * S; y < yb * S; y++) {
                  for (int x = xa * S; x < xb * S; x++) {
                    int sum_sq = 0;
                    for (int k = 0; k < 4; k++) {
                      sum_sq += sqr(colors[y][x][k] - paint_into[k]);
                    }
                    diff += SQRT[sum_sq];
                  }
                  if (penalty + llround(diff * 5) >= ft) {
                    break;
                  }
                }
                penalty += llround(diff * 5);
                ft = min(ft, (int) penalty);
              }
            }
            dp[aux[xa][xb]][aux[ya][yb]] = ft;
          }
        }
      }
    }
    msg = "dp = " + to_string(dp[aux[0][n]][aux[0][m]] / 1000);
    vector<pair<array<int, 4>, Color>> rects;
    vector<vector<int>> rect_id(n, vector<int>(m, -1));
    function<void(int, int, int, int)> Reconstruct = [&](int xa, int ya, int xb, int yb) {
      int ft = dp[aux[xa][xb]][aux[ya][yb]];
      for (int x = xa + 1; x < xb; x++) {
        if (dp[aux[xa][x]][aux[ya][yb]] + dp[aux[x][xb]][aux[ya][yb]] == ft) {
          Reconstruct(xa, ya, x, yb);
          Reconstruct(x, ya, xb, yb);
          return;
        }
      }
      for (int y = ya + 1; y < yb; y++) {
        if (dp[aux[xa][xb]][aux[ya][y]] + dp[aux[xa][xb]][aux[y][yb]] == ft) {
          Reconstruct(xa, ya, xb, y);
          Reconstruct(xa, y, xb, yb);
          return;
        }
      }
      int area = (xb - xa) * (yb - ya) * S * S;
      Color paint_into;
      for (int k = 0; k < 4; k++) {
        int sum = pref[yb * S][xb * S][k] - pref[ya * S][xb * S][k] - pref[yb * S][xa * S][k] + pref[ya * S][xa * S][k];
        paint_into[k] = (2 * sum + area) / (2 * area);
      }
      for (int x = xa; x < xb; x++) {
        for (int y = ya; y < yb; y++) {
          rect_id[x][y] = (int) rects.size();
        }
      }
      rects.emplace_back(array<int, 4>{xa, ya, xb, yb}, paint_into);
    };
    Reconstruct(0, 0, n, m);
    Solution res;
    res.score = dp[aux[0][n]][aux[0][m]] / 1000;
    int rect_cnt = (int) rects.size();
    vector<vector<int>> graph(rect_cnt);
    vector<int> indegree(rect_cnt);
    auto AddEdge = [&](int i, int j) {
      if (i != j) {
        graph[i].push_back(j);
        indegree[j] += 1;
      }
    };
    for (int x = 0; x < n; x++) {
      for (int y = 0; y < m - 1; y++) {
        if (mode & 2) {
          AddEdge(rect_id[x][y + 1], rect_id[x][y]);
        } else {
          AddEdge(rect_id[x][y], rect_id[x][y + 1]);
        }
      }
    }
    for (int x = 0; x < n - 1; x++) {
      for (int y = 0; y < m; y++) {
        if (mode & 1) {
          AddEdge(rect_id[x + 1][y], rect_id[x][y]);
        } else {
          AddEdge(rect_id[x][y], rect_id[x + 1][y]);
        }
      }
    }
    vector<int> que;
    for (int i = 0; i < rect_cnt; i++) {
      if (indegree[i] == 0) {
        que.push_back(i);
      }
    }
    for (int b = 0; b < (int) que.size(); b++) {
      for (int u : graph[que[b]]) {
        if (--indegree[u] == 0) {
          que.push_back(u);
        }
      }
    }
    assert((int) que.size() == rect_cnt);
    auto Compare = [&](int x, int y) {
      int cand1 = llround(1.0 * (n * m) / (max(x, n - x) * y));
      cand1    += llround(1.0 * (n * m) / (max(x, n - x) * (m - y)));
      cand1    += llround(1.0 * m / max(y, m - y));
      int cand2 = llround(1.0 * (n * m) / (x * max(y, m - y)));
      cand2    += llround(1.0 * (n * m) / ((n - x) * max(y, m - y)));
      cand2    += llround(1.0 * n / max(x, m - x));
      return cand1 < cand2;
    };
    int idx = 0;
    for (int it = 0; it < rect_cnt; it++) {
      int i = que[it];
      int xa = rects[i].first[0];
      int ya = rects[i].first[1];
      int xb = rects[i].first[2];
      int yb = rects[i].first[3];
      Color paint_into = rects[i].second;
      if (mode == 0) {
        if (xa == 0 && ya == 0) {
          res.ins.push_back(ColorIns(to_string(idx), paint_into));
        }
        if (xa == 0 && ya > 0) {
          res.ins.push_back(SplitYIns(to_string(idx), ya * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".0", paint_into));
          res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          idx += 1;
        }
        if (xa > 0 && ya == 0) {
          res.ins.push_back(SplitXIns(to_string(idx), xa * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".1", paint_into));
          res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          idx += 1;
        }
        if (xa > 0 && ya > 0) {                   
          res.ins.push_back(SplitPointIns(to_string(idx), xa * S, ya * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".1", paint_into));
          if (Compare(n - xa, n - ya)) {
            res.ins.push_back(MergeIns(to_string(idx) + ".3", to_string(idx) + ".2"));
            res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          } else {
            res.ins.push_back(MergeIns(to_string(idx) + ".3", to_string(idx) + ".0"));
            res.ins.push_back(MergeIns(to_string(idx) + ".2", to_string(idx) + ".1"));
          }
          res.ins.push_back(MergeIns(to_string(idx + 1), to_string(idx + 2)));
          idx += 3;
        }
      }
      if (mode == 1) {
        if (xb == n && ya == 0) {
          res.ins.push_back(ColorIns(to_string(idx), paint_into));
        }
        if (xb == n && ya > 0) {
          res.ins.push_back(SplitYIns(to_string(idx), ya * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".0", paint_into));
          res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          idx += 1;
        }
        if (xb < n && ya == 0) {
          res.ins.push_back(SplitXIns(to_string(idx), xb * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".0", paint_into));
          res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          idx += 1;
        }
        if (xb < n && ya > 0) {                   
          res.ins.push_back(SplitPointIns(to_string(idx), xb * S, ya * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".0", paint_into));
          if (Compare(xb, n - ya)) {
            res.ins.push_back(MergeIns(to_string(idx) + ".3", to_string(idx) + ".2"));
            res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          } else {
            res.ins.push_back(MergeIns(to_string(idx) + ".3", to_string(idx) + ".0"));
            res.ins.push_back(MergeIns(to_string(idx) + ".2", to_string(idx) + ".1"));
          }
          res.ins.push_back(MergeIns(to_string(idx + 1), to_string(idx + 2)));
          idx += 3;
        }
      }
      if (mode == 2) {
        if (xa == 0 && yb == n) {
          res.ins.push_back(ColorIns(to_string(idx), paint_into));
        }
        if (xa == 0 && yb < n) {
          res.ins.push_back(SplitYIns(to_string(idx), yb * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".1", paint_into));
          res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          idx += 1;
        }
        if (xa > 0 && yb == n) {
          res.ins.push_back(SplitXIns(to_string(idx), xa * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".1", paint_into));
          res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          idx += 1;
        }
        if (xa > 0 && yb < n) {                   
          res.ins.push_back(SplitPointIns(to_string(idx), xa * S, yb * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".2", paint_into));
          if (Compare(n - xa, yb)) {
            res.ins.push_back(MergeIns(to_string(idx) + ".3", to_string(idx) + ".2"));
            res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          } else {
            res.ins.push_back(MergeIns(to_string(idx) + ".3", to_string(idx) + ".0"));
            res.ins.push_back(MergeIns(to_string(idx) + ".2", to_string(idx) + ".1"));
          }
          res.ins.push_back(MergeIns(to_string(idx + 1), to_string(idx + 2)));
          idx += 3;
        }
      }
      if (mode == 3) {
        if (xb == n && yb == n) {
          res.ins.push_back(ColorIns(to_string(idx), paint_into));
        }
        if (xb == n && yb < n) {
          res.ins.push_back(SplitYIns(to_string(idx), yb * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".1", paint_into));
          res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          idx += 1;
        }
        if (xb < n && yb == n) {
          res.ins.push_back(SplitXIns(to_string(idx), xb * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".0", paint_into));
          res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          idx += 1;
        }
        if (xb < n && yb < n) {                   
          res.ins.push_back(SplitPointIns(to_string(idx), xb * S, yb * S));
          res.ins.push_back(ColorIns(to_string(idx) + ".3", paint_into));
          if (Compare(xb, yb)) {
            res.ins.push_back(MergeIns(to_string(idx) + ".3", to_string(idx) + ".2"));
            res.ins.push_back(MergeIns(to_string(idx) + ".0", to_string(idx) + ".1"));
          } else {
            res.ins.push_back(MergeIns(to_string(idx) + ".3", to_string(idx) + ".0"));
            res.ins.push_back(MergeIns(to_string(idx) + ".2", to_string(idx) + ".1"));
          }
          res.ins.push_back(MergeIns(to_string(idx + 1), to_string(idx + 2)));
          idx += 3;
        }
      }
    }

    msg = "Duration: " + to_string(GetTime()) + "s\n";
    postprocess(res);
}

void optsWindow() {
    if (ImGui::Begin("Solution")) {
        ImGui::Text("Current test: %d", currentTestId);
        ImGui::Checkbox("SplitX", &useSplitX);
        ImGui::Checkbox("SplitY", &useSplitY);
        ImGui::Checkbox("SplitPoint", &useSplitPoint);
        ImGui::DragInt("DP Step", &S, 1, 2, 200, "S=%d", ImGuiSliderFlags_AlwaysClamp);
        ImGui::DragInt("Direction", &mode, 1, 0, 3, "D=%d", ImGuiSliderFlags_AlwaysClamp);

        if (ImGui::Button("Solve DP")) {
            cerr << "Spawn thread!\n";
            thread solveThread(solveDP);
            solveThread.detach();
        }
        if (ImGui::Button("Solve Gena")) {
            if (0) {
                cerr << "Run in main thread!\n";
                solveGena();
            } else {
                cerr << "Spawn thread!\n";
                thread solveThread(solveGena);
                solveThread.detach();
            }
        }
        ImGui::Text("%s\n%s", msg.c_str(), requestResult.c_str());
    }
    ImGui::End();
}

void inputWindow() {
    auto& io = ImGui::GetIO();
    if (ImGui::Begin("Mouse & Keyboard")) {
        if (ImGui::IsMousePosValid())
            ImGui::Text("Mouse pos: (%g, %g)", io.MousePos.x, io.MousePos.y);
        else
            ImGui::Text("Mouse pos: <INVALID>");
        ImGui::Text("Mouse delta: (%g, %g)", io.MouseDelta.x, io.MouseDelta.y);

        int count = IM_ARRAYSIZE(io.MouseDown);
        ImGui::Text("Mouse down:");         for (int i = 0; i < count; i++) if (ImGui::IsMouseDown(i))      { ImGui::SameLine(); ImGui::Text("b%d (%.02f secs)", i, io.MouseDownDuration[i]); }
        ImGui::Text("Mouse clicked:");      for (int i = 0; i < count; i++) if (ImGui::IsMouseClicked(i))   { ImGui::SameLine(); ImGui::Text("b%d (%d)", i, ImGui::GetMouseClickedCount(i)); }
        ImGui::Text("Mouse released:");     for (int i = 0; i < count; i++) if (ImGui::IsMouseReleased(i))  { ImGui::SameLine(); ImGui::Text("b%d", i); }
        ImGui::Text("Mouse wheel: %.1f", io.MouseWheel);

        ImGui::Separator();

        const ImGuiKey key_first = ImGuiKey_NamedKey_BEGIN;
        ImGui::Text("Keys down:");          for (ImGuiKey key = key_first; key < ImGuiKey_COUNT; key++) { if (ImGui::IsKeyDown(key)) { ImGui::SameLine(); ImGui::Text("\"%s\" %d", ImGui::GetKeyName(key), key); } }
    }
    ImGui::End();
}

int main(int, char**)
{
    running = true;
    for (int i = 1; i < 1000; i++) myScores[i] = -1;
    thread updateThread(updateStandingsTimed);
    SDLWrapper sw;
    if (!sw.init()) return -1;

    while (true) {
        if (sw.checkQuit()) break;
        sw.newFrame();
        // ImGui::GetIO().FontGlobalScale = 1.5;

        // inputWindow();
        fileWindow();
        optsWindow();
        
        processMouse();
        draw();

        sw.finishFrame();
    }

    sw.cleanup();

    running = false;
    updateThread.join();
    return 0;
}
