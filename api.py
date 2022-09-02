import requests, sys

headers = {
    'authorization': 'Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJlbWFpbCI6Im1haWxyb21rYUBnbWFpbC5jb20iLCJleHAiOjE2NjIyMDc1ODksIm9yaWdfaWF0IjoxNjYyMTIxMTg5fQ.euzTfElCK7Jhuu-s3EgjxDIbDxct6yNrwSvxwSC9IJM'
}

submit_url = 'https://robovinci.xyz/api/submissions/{}/create'
submissions_url = 'https://robovinci.xyz/api/submissions'
scoreboard_url = 'https://robovinci.xyz/api/results/scoreboard'

def submit(task_id, fname):
    contents = open(fname, "rb").read()
    rs = requests.post(submit_url.format(task_id), headers=headers, files={'file': contents})
    print(rs.text)

def save_standings():
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
            fout.write("{} {} {}\n".format(score, -solved, team))
            t = team
            if t == 'RGBTeam':
                t = '--> RGBTeam <--'
                myresult = score
            if len(t) > 20:
                t = t[:17] + "..."

            cnt += 1
            if cnt <= 20 or team == 'RGBTeam':
                print("{3:2d} {0:20s} {1:2d} {2}".format(t, -solved, score, cnt))

    mytest = {}
    mintest = {}
    for team in js['users']:
        for test in team['results']:
            if test['submission_count'] == 0:
                continue
            tid = test['problem_id']
            if tid not in mintest:
                mintest[tid] = 10 ** 10
            mintest[tid] = min(mintest[tid], test['min_cost'])
            if team['team_name'] == 'RGBTeam':
                mytest[tid] = test['min_cost']

    with open("tests.txt", "w") as fout:
        print('===== Tests =====')
        min_total = 0
        for tid in sorted(mintest.keys()):
            fout.write(f"{tid} {mytest[tid]} {mintest[tid]}\n")
            print("{0:2d} {1:8d}:our {2:8d}:best {3:8d}:loss".format(tid, mytest[tid], mintest[tid], mytest[tid] - mintest[tid]))
            min_total += mintest[tid]
        print(f"Sum of best results: {min_total}, our loss: {myresult - min_total}")


if __name__ == "__main__":
    if sys.argv[1] == 'standings':
        save_standings()

    if sys.argv[1] == 'submit':
        submit(sys.argv[2], sys.argv[3])
