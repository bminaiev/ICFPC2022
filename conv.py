from PIL import Image
import os

image_dir = "images/"
inputs_dir = "inputs/"
for f in sorted(os.listdir(image_dir)):
    fname = image_dir + f
    print(fname + "...")
    img = Image.open(fname)
    w, h = img.size
    with open(inputs_dir + f.replace("png", "txt"), "w") as fout:
        fout.write(f"{h} {w}\n")
        for y in range(h):
            for x in range(w):
                fout.write(" ".join(map(str, img.getpixel((x, y)))) + " ")
            fout.write("\n")

