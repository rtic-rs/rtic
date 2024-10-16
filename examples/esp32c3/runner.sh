#!/bin/bash

if [ $# -eq 0 ]; then
  echo "No arguments supplied! Provide path to ELF as argument"
fi

outputfilenamecargo=$1
outputfilename="$outputfilenamecargo".bin

logfile=qemu.log

qemuexec=qemu-system-riscv32

# Building ESP32-C3 image
espflash save-image --chip esp32c3 --merge "$outputfilenamecargo" "$outputfilename" 1>&2

# Get stats
esptool.py image_info --version 2 "$outputfilename" 1>&2

# Run in QEMU
$qemuexec -nographic -monitor tcp:127.0.0.1:55555,server,nowait -icount 3 -machine esp32c3 -drive file="$outputfilename",if=mtd,format=raw -serial file:"$logfile" &

# Let it run
sleep 3s

# Kill QEMU nicely by sending 'q' (quit) over tcp
echo q | nc -N 127.0.0.1 55555

# Output that will be compared must be printed to stdout

# Make boot phase silent, for debugging change, run with e.g.  $ `env DEBUGGING=true` cargo xtask....
if [ -n "${DEBUGGING}" ]; then
  # Debugging: strip leading "I (xyz)" where xyz is an incrementing number, and esp_image specifics
  sed -e 's/esp_image: .*$/esp_image: REDACTED/' -e 's/I\s\([0-9]*\)(.*)/\1/' < $logfile
else
  tail -n +12 "$logfile" | sed -e '/I\s\([0-9]*\)(.*)/d'
fi
