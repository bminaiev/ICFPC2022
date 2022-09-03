
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

#include <list>
#include <set>
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

using namespace std;
typedef std::chrono::high_resolution_clock Time;
typedef std::chrono::milliseconds chrono_ms;
    
#define forn(i, N) for (int i = 0; i < (int)(N); i++)
#define sqr(x) (x)*(x)
using ll = long long;
using Color = array<int, 4>;


#include "api.h"
#include "solutions.h"

namespace fs = std::filesystem;
const string inputsPath = "../inputs/";
const string solutionsPath = "../solutions/";


int selected_idx, currentTestId;
Painter painter;
unordered_map<int, int> myScores;

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

void postprocess(Solution& res) {
    painter = Painter(N, M);
    msg += "Solved with penalty " + to_string(res.score) + "\n";
    for (const auto& ins : res.ins) {
        if (!painter.doInstruction(ins)) {
            msg += "Bad instruction: " + ins.text() + "\n";
            return;
        }
    }
    msg += "Painter score: " + to_string(painter.totalScore(colors)) + "\n";
    res.score = painter.totalScore(colors);
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
    std::ifstream ifs("local_scores.txt");
    int test_id, score;
    while (ifs >> test_id >> score) {
        myScores[test_id] = score;
    }
}

chrono::high_resolution_clock::time_point lastUpdateTime;

void updateStandingsTimed() {
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
                bName = "Sub " + to_string(tid);
                if (ImGui::Button(bName.c_str())) {
                    apiSubmit(tests[idx].first);
                }
                
                if (idx < testResults.size()) {
                    assert(get<0>(testResults[idx]) == tests[idx].first);
                    ImGui::TableNextColumn();
                    ImGui::Text("%d", get<1>(testResults[idx]));
                    ImGui::SameLine(55);
                    bName = "DL " + to_string(tid);
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

void optsWindow() {
    static bool runInMainThread = false;
    if (ImGui::Begin("Solution")) {
        ImGui::Text("Current test: %d", currentTestId);
        if (currentTestId >= 1) {
            ImGui::DragInt("DP Step", &S, 1, 2, 200, "S=%d", ImGuiSliderFlags_AlwaysClamp);
            ImGui::DragInt("Direction", &mode, 1, 0, 3, "D=%d", ImGuiSliderFlags_AlwaysClamp);
            ImGui::Checkbox("Run in main thread", &runInMainThread);

            if (ImGui::Button("Solve Gena")) {
                if (runInMainThread) {
                    cerr << "Run in main thread!\n";
                    solveGena(S, mode);
                } else {
                    cerr << "Spawn thread!\n";
                    thread solveThread(solveGena, S, mode);
                    solveThread.detach();
                }
            }
            ImGui::SameLine(100);
            if (ImGui::Button("Solve Opt")) {
                if (runInMainThread) {
                    cerr << "Run in main thread!\n";
                    solveOpt();
                } else {
                    cerr << "Spawn thread!\n";
                    thread solveThread(solveOpt);
                    solveThread.detach();
                }
            }        
            ImGui::InputInt("TL, sec", &optSeconds, 1, 10);

            ImGui::Text("%s\n%s", msg.c_str(), requestResult.c_str());
        }
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
