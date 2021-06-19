#!/bin/bash

# Assemble the program
./utils/as0 tests/test.asm -l cre c s

# Extract the binary from the S19 record
objcopy --input-target=srec --output-target=binary tests/test.s19 tests/test.bin
