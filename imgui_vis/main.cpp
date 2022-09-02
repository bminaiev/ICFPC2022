
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

int N, M;

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
            sprintf(buf, "cut [%s] [Y] [%d]", id.c_str(), N - y);
        } else assert(false);
        return buf;
    }
};

Instruction ColorIns(string i, Color c) {
    Instruction res;
    res.type = tColor;
    res.id = i;
    res.color = c;
    return res;
}

Instruction SplitPointIns(string i, int x, int y) {
    Instruction res;
    res.type = tSplitPoint;
    res.id = i;
    res.x = x;
    res.y = y;
    return res;
}

Instruction SplitXIns(string i, int x) {
    Instruction res;
    res.type = tSplitX;
    res.id = i;
    res.x = x;
    return res;
}

Instruction SplitYIns(string i, int y) {
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

struct Block {
    int r1, c1, r2, c2;
};

struct Painter {
    int lastBlockId;
    int N, M;
    unordered_map<string, Block> blocks;
    vector<vector<Color>> colors;

    Painter() {}
    Painter(int n, int m) {
        lastBlockId = 0;
        N = n;
        M = m;
        Color c;
        c[0] = c[1] = c[2] = c[3] = 255;
        colors.assign(n, vector<Color>(m, c));
        for (int i = 0; i < n; i++)
            for (int j = 0; j < m; j++)
                colors[i][j] = c;
        blocks["0"] = Block{0, 0, N, M};
    }

    bool doColor(string i, Color c) {
        if (blocks.find(i) == blocks.end())
            return false;
        const auto& b = blocks[i];
        for (int i = b.r1; i < b.r2; i++)
            for (int j = b.c1; j < b.c2; j++)
                colors[i][j] = c;
        return true;
    }

    bool doSplitX(string i, int x) {
        if (blocks.find(i) == blocks.end())
            return false;
        const auto& b = blocks[i];
        if (x <= b.c1 || x >= b.c2) return false;
        Block left = b;
        Block right = b;
        blocks.erase(blocks.find(i));
        left.c2 = x;
        right.c1 = x;
        blocks[i + ".0"] = left;
        blocks[i + ".1"] = right;
        return true;
    }

    bool doSplitY(string i, int y) {
        if (blocks.find(i) == blocks.end())
            return false;
        const auto& b = blocks[i];
        if (y <= b.r1 || y >= b.r2) return false;
        Block down = b;
        Block up = b;
        blocks.erase(blocks.find(i));
        down.r1 = y;
        up.r2 = y;
        blocks[i + ".0"] = down;
        blocks[i + ".1"] = up;
        return true;
    }

    bool doSplitPoint(string i, int x, int y) {
        if (blocks.find(i) == blocks.end())
            return false;
        const auto& b = blocks[i];
        if (x <= b.c1 || x >= b.c2) return false;
        if (y <= b.r1 || y >= b.r2) return false;
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

    bool doInstruction(const Instruction& ins) {
        if (ins.type == tColor) {
            return doColor(ins.id, ins.color);            
        } else if (ins.type == tSplitPoint) {
            return doSplitPoint(ins.id, ins.x, ins.y);
        } else if (ins.type == tSplitX) {
            return doSplitX(ins.id, ins.x);
        } else if (ins.type == tSplitY) {
            return doSplitY(ins.id, ins.y);
        } else return false;
    }
};

int selected_idx, test_id;
vector<vector<Color>> colors;
unordered_map<ll, Solution> mem;
int S = 10;
Painter painter;

double scale = 1;
double shiftX, shiftY;


void readInput(const string& fname) {
    freopen(fname.c_str(), "r", stdin);
    cin >> N >> M;
    colors.assign(N, vector<Color>(M, Color()));
    for (int i = 0; i < N; i++)
        for (int j = 0; j < M; j++)
            for (int q = 0; q < 4; q++)
                cin >> colors[i][j][q];
    scale = 1;
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

            if (i < painter.colors.size() && j < painter.colors[i].size()) {
                color = IM_COL32(painter.colors[i][j][0], painter.colors[i][j][1],
                                 painter.colors[i][j][2], painter.colors[i][j][3]);
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

Solution getInstructions(string id, int r1, int c1, int r2, int c2) {
    ll key = ((r1 * M + c1) * ll(N) + r2) * ll(M) + c2;
    if (mem.find(key) != mem.end()) {
        auto res = mem[key];
        for (size_t w = 0; w < res.ins.size(); w++)
            res.ins[w].id = id + res.ins[w].id;
        return res;
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

    // cerr << r1 << "," << c1 << " " << r2 << "," << c2 << " - ";
    // for (const auto& i : res.ins) cerr << " " << i.text();
    // cerr << endl;
    mem[key] = res;
    for (size_t w = 0; w < res.ins.size(); w++)
        res.ins[w].id = id + res.ins[w].id;
    return res;
}

string msg;

Solution solveDP() {
    mem.clear();
    Solution res = getInstructions("0", 0, 0, N, M);
    painter = Painter(N, M);
    msg = "Solved with penalty " + to_string(res.score) + "\n";
    for (const auto& ins : res.ins) {
        if (!painter.doInstruction(ins)) {
            msg += "Bad instruction: " + ins.text() + "\n";
            res.score = -1;
            return res;
        }
    }
    return res;
}

void optsWindow() {
    if (ImGui::Begin("Solution")) {
        ImGui::SliderInt("DP Cell Size", &S, 4, 200);
        if (ImGui::Button("Solve DP")) {
            Solution res = solveDP();
            if (res.score != -1) {
                string fname = "../solutions/" + to_string(test_id) + ".txt";
                freopen(fname.c_str(), "w", stdout);
                for (const auto& i : res.ins) {
                    cout << i.text() << endl;
                }
                fclose(stdout);
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
