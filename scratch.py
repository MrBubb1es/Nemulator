some_bytes = b'NES\x1A\x15\x54\x4E\x73\xB1\xAE\x04\xF0\xFE\x76\x22\xF9'
 
for i in range(10000):
    some_bytes += b'\x00'

# Open in "wb" mode to
# write a new file, or 
# "ab" mode to append
with open("prg_tests/cpu_tests/my_file.bin", "wb") as binary_file:
   
    # for byte in binary_file:
    #     print(byte)

    # Write bytes to file
    binary_file.write(some_bytes)
