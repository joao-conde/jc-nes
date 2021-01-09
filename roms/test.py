nestestlogs = open("roms/nestest.log")
mylogs = open("roms/mylogs.txt")

for (nestest_line, my_line) in zip(nestestlogs, mylogs):
    expected = nestest_line.split(" A:")
    expected = [
        expected[0].split()[0],
        expected[0].split()[1],
        "A:" + expected[1].split()[0],
        expected[1].split()[1],
        expected[1].split()[2],
        expected[1].split()[3],
        expected[1].split()[4],
        expected[1].split()[-1],
    ]
    
    mine = my_line.split()

    if expected != mine:
        print("expected:", expected)
        print("mine    :", mine)
        assert(False)


nestestlogs.close()
mylogs.close()
