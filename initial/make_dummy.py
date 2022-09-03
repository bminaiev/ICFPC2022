for i in range(1, 26):
    with open(f"{i}.json", "w") as out:
        out.write("""{
    "width": 400,
    "height": 400,
    "blocks": [
        {
            "blockId": "0",
            "bottomLeft": [0, 0],
            "topRight": [400, 400],
            "color": [255, 255, 255, 255]
        }
    ]
}""")
