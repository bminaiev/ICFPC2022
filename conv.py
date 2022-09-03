from PIL import Image
import os, json

image_dir = "images/"
inputs_dir = "inputs/"
jsons_dir = "initial/"
for f in sorted(os.listdir(image_dir), key=lambda s: int(s[:s.find('.')])):
    if 'initial' in f:
        continue
    fname = image_dir + f
    print(fname + "...")
    img = Image.open(fname)
    w, h = img.size
    js = json.load(open(jsons_dir + f.replace("png", "json")))
    assert(w == js['width'])
    assert(h == js['height'])
    with open(inputs_dir + f.replace("png", "txt"), "w") as fout:
        fout.write(f"{h} {w}\n")
        for y in range(h):
            for x in range(w):
                fout.write(" ".join(map(str, img.getpixel((x, h - 1 - y)))) + " ")
            fout.write("\n")

        fout.write(f"{len(js['blocks'])}\n")
        for i, b in enumerate(js['blocks']):
            assert(int(b['blockId']) == i)
            fout.write(f"{b['blockId']} " + " ".join(map(str, b['bottomLeft'] + b["topRight"] + b["color"])) + "\n")
