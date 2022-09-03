import requests, sys, shutil, json

headers = {
    'authorization': 'Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6Im1haWxyb21rYUBnbWFpbC5jb20iLCJleHAiOjE2NjIyMDc1ODksIm9yaWdfaWF0IjoxNjYyMTIxMTg5fQ.euzTfElCK7Jhuu-s3EgjxDIbDxct6yNrwSvxwSC9IJM'
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


def save_standings():
    mytest = {}                
    rs = requests.get(submissions_url, headers=headers)
    js = rs.json()
    for sub in js['submissions']:
        if sub['status'] == 'SUCCEEDED':
            if sub['problem_id'] not in mytest:
                mytest[sub['problem_id']] = 10 ** 10

            mytest[sub['problem_id']] = min(mytest[sub['problem_id']], sub['score'])

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

    mintest = {}
    for team in js['users']:
        for test in team['results']:
            tid = test['problem_id']
            if tid not in mintest:
                mintest[tid] = 10 ** 9
            if test['submission_count'] == 0:
                continue
            mintest[tid] = min(mintest[tid], test['min_cost'])
            # if tid == 1:
            #     print(test['min_cost'])

    with open("tests.txt", "w") as fout:
        print('===== Tests =====')
        min_total = 0
        for tid in sorted(mintest.keys()):
            fout.write(f"{tid} {mytest[tid]} {mintest[tid]}\n")
            print("{0:2d} {1:8d}:our {2:8d}:best {3:8d}:{4}".format(tid, mytest[tid], mintest[tid],
                mytest[tid] - mintest[tid], "loss" if mytest[tid] > mintest[tid] else "win"))
            min_total += mintest[tid]
        print(f"Sum of best results: {min_total}, Our results: {myresult}, Loss: {myresult - min_total}")


if __name__ == "__main__":
    if sys.argv[1] == 'standings':
        save_standings()

    if sys.argv[1] == 'submit':
        submit(sys.argv[2], sys.argv[3])

    if sys.argv[1] == 'download':
        download(int(sys.argv[2]), sys.argv[3])