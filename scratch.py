a = """1 1111 (1F) => 30
1 1101 (1D) => 28
1 1011 (1B) => 26
1 1001 (19) => 24
1 0111 (17) => 22
1 0101 (15) => 20
1 0011 (13) => 18
1 0001 (11) => 16
0 1111 (0F) => 14
0 1101 (0D) => 12
0 1011 (0B) => 10
0 1001 (09) => 8
0 0111 (07) => 6
0 0101 (05) => 4
0 0011 (03) => 2
0 0001 (01) => 254

Notes with base length 12 (4/4 at 75 bpm):
1 1110 (1E) => 32  (96 times 1/3, quarter note triplet)
1 1100 (1C) => 16  (48 times 1/3, eighth note triplet)
1 1010 (1A) => 72  (48 times 1 1/2, dotted quarter)
1 1000 (18) => 192 (Whole note)
1 0110 (16) => 96  (Half note)
1 0100 (14) => 48  (Quarter note)
1 0010 (12) => 24  (Eighth note)
1 0000 (10) => 12  (Sixteenth)

Notes with base length 10 (4/4 at 90 bpm, with relative durations being the same as above):
0 1110 (0E) => 26  (Approx. 80 times 1/3, quarter note triplet)
0 1100 (0C) => 14  (Approx. 40 times 1/3, eighth note triplet)
0 1010 (0A) => 60  (40 times 1 1/2, dotted quarter)
0 1000 (08) => 160 (Whole note)
0 0110 (06) => 80  (Half note)
0 0100 (04) => 40  (Quarter note)
0 0010 (02) => 20  (Eighth note)
0 0000 (00) => 10  (Sixteenth)"""


lookup_list = []

for line in a.split('\n'):
    if len(line) == 0:
        continue

    if line[0] == '0' or line[0] == '1':
        idx_num = line[0:6].replace(' ', '')
        idx_num = int(idx_num, base=2)

        start = line.find("=> ") + 2
        end = line.find(" (", start) + 1

        lookup_list.append((idx_num, line))

lookup_list.sort(key=lambda x: x[0])

for thing in lookup_list:
    print(thing)