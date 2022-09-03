import requests, sys, shutil, json
from collections import defaultdict
from termcolor import colored

headers = {
    'authorization': 'Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6Im1haWxyb21rYUBnbWFpbC5jb20iLCJleHAiOjE2NjIyOTc1MzIsIm9yaWdfaWF0IjoxNjYyMjExMTMyfQ.Uz68RJBnRdKHnJHYGUwzYxYccg-wqNDPZITGSUhj1C8'
}

submit_url = 'https://robovinci.xyz/api/submissions/{}/create'
submissions_url = 'https://robovinci.xyz/api/submissions'
scoreboard_url = 'https://robovinci.xyz/api/results/scoreboard'

def submit(task_id, fname):
    contents = open(fname, "rb").read()
    with open("req_result.txt", "w") as fout:
        rs = requests.post(submit_url.format(task_id), headers=headers, files={'file': contents})
        fout.write(rs.text)

def download(task_id, fname):
    rs = requests.get(submissions_url, headers=headers)
    js = rs.json()
    best = (10 ** 100, 0)
    for sub in js['submissions']:
        if sub['status'] == 'SUCCEEDED' and sub['problem_id'] == task_id:
            best = min(best, (sub['score'], sub['id']))

    rs = requests.get(submissions_url + "/" + str(best[1]), headers=headers)
    print(rs.text)
    js = rs.json()
    url = js['file_url']

    r = requests.get(url, stream=True)
    if r.status_code == 200:
        with open(fname, 'wb') as f:
            r.raw.decode_content = True
            shutil.copyfileobj(r.raw, f)

    with open("req_result.txt", "w") as fout:
        js['file_url'] = '<stripped>'
        fout.write(json.dumps(js))


def print_ultra_cool_tests(test_results):
    top_list = defaultdict(list)
    my, opp = {}, {}
    for score, test_id, team in test_results:
        if len(top_list[test_id]) == 5:
            continue

        for p in top_list[test_id]:
            if p[1] == team:
                break
        else:
            if team != 'RGBTeam-local' or all(x[1] != 'RGBTeam' for x in top_list[test_id]):
                top_list[test_id].append((score, team))

        my[test_id] = 10 ** 10
        opp[test_id] = 10 ** 10

    for score, test_id, team in test_results[::-1]:
        if team.startswith('RGBTeam'):
            my[test_id] = score
        else:
            opp[test_id] = score

    cur = 1
    cols = 6
    col_width = 30
    def pad(text):
        if len(text) > col_width - 2:
            text = text[:20] + "..."
        text = " " * ((col_width - len(text)) // 2) + text
        text += " " * (col_width - len(text))
        return text
    while cur <= max(top_list.keys()):
        print("-" * (col_width * cols + cols + 1))
        sys.stdout.write("|")
        for j in range(cols):
            if top_list[cur+j] and top_list[cur+j][0][1].startswith('RGBTeam'):
                sys.stdout.write(colored(pad(f"Test {cur+j} (Adv. {my[cur+j] - opp[cur+j]})"), 'green'))
            else:
                sys.stdout.write(colored(pad(f"Test {cur+j} (Loss {my[cur+j] - opp[cur+j]})"), 'red'))
            sys.stdout.write("|")
        print()
        print((" " * col_width).join("|" * (cols + 1)))

        for p in range(4):
            sys.stdout.write("|")
            for j in range(cols):
                tl = top_list[cur + j]
                if p >= len(tl):
                    sys.stdout.write(" " * col_width)
                else:
                    if tl[p][1] == 'RGBTeam':
                        sys.stdout.write(colored(pad(f"{tl[p][0]} {tl[p][1]}"), 'yellow'))
                    elif tl[p][1] == 'RGBTeam-local':
                        sys.stdout.write(colored(pad(f"{tl[p][0]} {tl[p][1]}"), 'cyan'))
                    else:
                        sys.stdout.write(pad(f"{tl[p][0]} {tl[p][1]}"))
                sys.stdout.write("|")
            print()

        cur += cols
    print("-" * (col_width * cols + cols + 1))



def save_standings():
    mytest = {}                
    rs = requests.get(submissions_url, headers=headers)
    js = rs.json()
    test_results = []
    for sub in js['submissions']:
        if sub['status'] == 'SUCCEEDED':
            if sub['problem_id'] not in mytest:
                mytest[sub['problem_id']] = 10 ** 10

            mytest[sub['problem_id']] = min(mytest[sub['problem_id']], sub['score'])
            test_results.append((sub['score'], sub['problem_id'], 'RGBTeam-local'))

    myresult = sum(mytest.values())

    rs = requests.get(scoreboard_url, headers=headers)
    with open("standings.txt", "w") as fout:
        # fout.write(rs.text)
        js = rs.json()
        standings = []
        for team in js['users']:
            standings.append((-team['solved_problem_count'], team['total_cost'], team['team_name']))
        print('===== Scoreboard =====')
        cnt = 0
        for solved, score, team in sorted(standings):
            t = team
            if t == 'RGBTeam':
                t = '--> RGBTeam <--'
            if len(t) > 20:
                t = t[:17] + "..."

            cnt += 1
            if cnt <= 20 or team == 'RGBTeam':
                line = "{3:2d} {0:20s} {1:2d} {2}".format(t, -solved, score, cnt)
                print(line)
                fout.write(line + "\n")

    mintest = defaultdict(list)
    for team in js['users']:
        for test in team['results']:
            tid = test['problem_id']
            if test['submission_count'] == 0:
                continue
            mintest[tid].append(test['min_cost'])
            test_results.append((test['min_cost'], tid, team['team_name']))



    with open("tests.txt", "w") as fout:
        print('===== Tests =====')
        min_total = 0
        for tid in sorted(mintest.keys()):
            mintest[tid].sort()
            while len(mintest[tid]) < 2:
                mintest[tid].append(10 ** 9)

            fout.write(f"{tid} {mytest[tid]} {mintest[tid][0]} {mintest[tid][1]}\n")
            print("{0:2d} {1:8d}:our {2:8d}:best {3:8d}:{4}".format(tid, mytest[tid], mintest[tid][0],
                mytest[tid] - mintest[tid][0] if mytest[tid] > mintest[tid][0] else mytest[tid] - mintest[tid][1],
                "loss" if mytest[tid] > mintest[tid][0] else "adv"))
            min_total += mintest[tid][0] if mintest[tid][0] < 10 ** 9 else 0
        print(f"Sum of best results: {min_total}, Our results: {myresult}, Loss: {myresult - min_total}")

    test_results.sort()
    print_ultra_cool_tests(test_results)


if __name__ == "__main__":
    if sys.argv[1] == 'standings':
        save_standings()

    if sys.argv[1] == 'submit':
        submit(sys.argv[2], sys.argv[3])

    if sys.argv[1] == 'download':
        download(int(sys.argv[2]), sys.argv[3])
