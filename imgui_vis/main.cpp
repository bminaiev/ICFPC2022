
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

#include <filesystem>
#include <iostream>
#include <cmath>
#include <vector>
#include <unordered_map>
#include <algorithm>
#include <array>
#include <cassert>

using namespace std;
namespace fs = std::filesystem;

#define forn(i, N) for (int i = 0; i < (int)(N); i++)
#define sqr(x) (x)*(x)
using ll = long long;
using Color = array<int, 4>;

constexpr int tColor = 1;
constexpr int tSplitPoint = 2;
constexpr int tSplitX = 3;
constexpr int tSplitY = 4;

struct Instruction {
    string id;
    int type;
    int x, y;
    Color color;

    string text() const {
        char buf[128];
        if (type == tColor) {
            sprintf(buf, "color [%s] [%d, %d, %d, %d]", id.c_str(), color[0], color[1], color[2], color[3]);
        } else if (type == tSplitPoint) {
            sprintf(buf, "cut [%s] [%d, %d]", id.c_str(), x, y);
        } else if (type == tSplitX) {
            sprintf(buf, "cut [%s] [X] [%d]", id.c_str(), x);
        } else if (type == tSplitY) {
            sprintf(buf, "cut [%s] [Y] [%d]", id.c_str(), y);
        } else assert(false);
        return buf;
    }
};

Instruction doColor(string i, Color c) {
    Instruction res;
    res.type = tColor;
    res.id = i;
    res.color = c;
    return res;
}

Instruction doSplitPoint(string i, int x, int y) {
    Instruction res;
    res.type = tSplitPoint;
    res.id = i;
    res.x = x;
    res.y = y;
    return res;
}

Instruction doSplitX(string i, int x) {
    Instruction res;
    res.type = tSplitX;
    res.id = i;
    res.x = x;
    return res;
}

Instruction doSplitY(string i, int y) {
    Instruction res;
    res.type = tSplitY;
    res.id = i;
    res.y = y;
    return res;
}

struct Solution {
    int score;
    vector<Instruction> ins;
};

int selected_idx, test_id;
int N, M;
vector<vector<Color>> colors;
unordered_map<ll, Solution> mem;
int S = 10;

double scale = 2;
double shiftX, shiftY;


void readInput(const string& fname) {
    freopen(fname.c_str(), "r", stdin);
    cin >> N >> M;
    colors.assign(N, vector<Color>(M, Color()));
    for (int i = 0; i < N; i++)
        for (int j = 0; j < M; j++)
            for (int q = 0; q < 4; q++)
                cin >> colors[i][j][q];
    scale = 1.5;
    shiftX = shiftY = 0;
}

void fileWindow() {
    if(ImGui::Begin("Tests")) {
        std::string path = "../inputs/";

        vector<pair<int, string>> tests;
        for (const auto & entry : fs::directory_iterator(path)) {
            string s = entry.path().string();
            tests.emplace_back(0, s);
            int i = 0;
            while (i < s.size() && (s[i] < '0' || s[i] > '9')) i++;
            if (i >= s.size()) continue;
            int j = i;
            while (s[j] >= '0' && s[j] <= '9') j++;
            sscanf(s.substr(i, j).c_str(), "%d", &tests.back().first);
        }

        sort(tests.begin(), tests.end());
        static int selected_idx = -1;

        if (ImGui::BeginListBox("T", ImVec2(250, ImGui::GetFrameHeightWithSpacing() * 16))) {
            for (size_t idx = 0; idx < tests.size(); idx++) {
                path = tests[idx].second;
                const bool is_selected = (idx == selected_idx);
                if (ImGui::Selectable(path.c_str(), is_selected)) {
                    selected_idx = idx;
                    test_id = tests[idx].first;
                    readInput(path);
                }

                // Set the initial focus when opening the combo (scrolling + keyboard navigation focus)
                if (is_selected)
                    ImGui::SetItemDefaultFocus();
            }
            ImGui::EndListBox();
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

Solution getInstructions(string id, int r1, int c1, int r2, int c2) {
    ll key = ((r1 * M + c1) * ll(N) + r2) * ll(M) + c2;
    if (mem.find(key) != mem.end()) {
        return mem[key];
    }
    cerr << r1 << "," << c1 << " " << r2 << "," << c2 << endl;

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

    int penalty = round(5.0 * N * M / ((r2 - r1) * (c2 - c1)));
    double colorPenalty = 0.0;
    for (int r = r1; r < r2; r++)
        for (int c = c1; c < c2; c++) {
            double ssq = 0;
            for (int q = 0; q < 4; q++)
                ssq += sqr(sum[q] - colors[r][c][q]);
            colorPenalty += sqrt(ssq);
        }
    penalty += round(colorPenalty * 0.005);

    Solution res;
    res.ins.push_back(doColor(id, sum));
    res.score = penalty;

    int cscore = round(7.0 * N * M / ((r2 - r1) * (c2 - c1)));
    for (int y = r1 + S; y < r2; y += S) {
        Solution s1 = getInstructions(id + ".1", r1, c1, y, c2);
        Solution s2 = getInstructions(id + ".0", y, c1, r2, c2);
        if (cscore + s1.score + s2.score < res.score) {
            res.score = cscore + s1.score + s2.score;
            res.ins.clear();
            res.ins.push_back(doSplitY(id, N - y));
            res.ins.insert(res.ins.end(), s1.ins.begin(), s1.ins.end());
            res.ins.insert(res.ins.end(), s2.ins.begin(), s2.ins.end());
        }
    }
/*
    for (int x = c1 + S; x < c2; x += S) {
        Solution s1 = getInstructions(id + ".0", r1, c1, r2, x);
        Solution s2 = getInstructions(id + ".1", r1, x, r2, c2);
        if (cscore + s1.score + s2.score < res.score) {
            res.score = cscore + s1.score + s2.score;
            res.ins.clear();
            res.ins.push_back(doSplitX(id, x - c1));
            res.ins.insert(res.ins.end(), s1.ins.begin(), s1.ins.end());
            res.ins.insert(res.ins.end(), s2.ins.begin(), s2.ins.end());
        }
    }
*/
    mem[key] = res;
    return res;
}

Solution solveDP() {
    mem.clear();
    Solution res = getInstructions("0", 0, 0, N, M);
    cerr << "here" << endl;
    return res;
}

void optsWindow() {
    static string msg;

    if (ImGui::Begin("Solution")) {
        ImGui::SliderInt("DP Cell Size", &S, 4, 200);
        if (ImGui::Button("Solve DP")) {
            Solution res = solveDP();
            msg = "Solved with penalty " + to_string(res.score);
            string fname = "../solutions/" + to_string(test_id) + ".txt";
            freopen(fname.c_str(), "w", stdout);
            for (const auto& i : res.ins) {
                cout << i.text() << endl;
            }
        }
        ImGui::Text("%s", msg.c_str());
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
    SDLWrapper sw;
    if (!sw.init()) return -1;

    while (true) {
        if (sw.checkQuit()) break;
        sw.newFrame();
        // ImGui::GetIO().FontGlobalScale = 1.5;

        inputWindow();
        fileWindow();
        optsWindow();
        
        processMouse();
        draw();

        sw.finishFrame();
    }

    sw.cleanup();
    return 0;
}
