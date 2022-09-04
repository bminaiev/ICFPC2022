import sys

id1 = int(sys.argv[1])
id2 = int(sys.argv[2])

fn1 = f"solutions/{id1}.txt"
fn2 = f"solutions/{id2}.txt"

lines1 = open(fn1).readlines()
lines2 = open(fn2).readlines()

i = 0
res = []
while not lines2[i].startswith("color"):
	res.append(lines2[i])
	i += 1

def replace_id(s, last_id):
	q = s.replace("[", "").replace("]", "")
	if '.' in q:
		t = q.split('.', 1)
		return f"[{int(t[0]) + last_id}.{t[1]}]"
	return f"[{int(q) + last_id}]"

last_id = int(lines2[i].replace("[", "").replace("]", "").split()[1])
res.append(lines2[i])
print(last_id)
assert(last_id != 0)
for line in lines1:
	tok = line.strip().split()
	if tok[0] == 'cut' or tok[0] == 'color':
		tok[1] = replace_id(tok[1], last_id)
	elif tok[0] == 'merge' or tok[0] == 'swap':
		tok[1] = replace_id(tok[1], last_id)
		tok[2] = replace_id(tok[2], last_id)
	res.append(" ".join(tok) + "\n")

with open(fn2, "w") as fout:
	for line in res:
		fout.write(line)