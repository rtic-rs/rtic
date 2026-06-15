#!/bin/bash

if [ $# -eq 0 ]; then
  echo "No arguments supplied! Provide path to ELF as argument"
  exit 1
fi

outputfilenamecargo=$1
outputfilename="$outputfilenamecargo".bin

logfile=qemu.log
qemuoutputfile=qemuoutput.log

qemuexec=/Users/vinayak/Developer/esp-qemu/qemu/bin/qemu-system-xtensa

tempdir=$(mktemp -d) || exit 1

# Build ESP32 flash image
espflash save-image --chip esp32 --merge "$outputfilenamecargo" "$outputfilename" 1>&2

# Run in QEMU
$qemuexec \
  -nographic \
  -monitor tcp:127.0.0.1:55556,server,nowait \
  -machine esp32 \
  -drive file="$outputfilename",if=mtd,format=raw \
  -serial file:"$tempdir/$logfile" \
  >"$tempdir/$qemuoutputfile" 2>&1 &

qemupid=$!

sleep 3s

echo q | nc -N 127.0.0.1 55556 >>"$tempdir/$qemuoutputfile" 2>&1

sleep 0.1s

pgrep -af "qemu-system-xtensa.*esp32.*" >/dev/null 2>&1 && echo q | nc -N 127.0.0.1 55556 >>"$tempdir/$qemuoutputfile" 2>&1

pgrep -af "qemu-system-xtensa.*esp32.*" >/dev/null 2>&1 && kill $qemupid >/dev/null 2>&1

pgrep -af "qemu-system-xtensa.*esp32.*" >/dev/null 2>&1 && sleep 0.1s

pgrep -af "qemu-system-xtensa.*esp32.*" >/dev/null 2>&1 && kill -9 $qemupid >/dev/null 2>&1

if [ -n "${DEBUGGING}" ]; then
  cat "$tempdir/$logfile"
else
  # ESP32 boot ROM prints several lines before user code runs; skip them
  tail -n +12 "$tempdir/$logfile" | sed -e '/I\s\([0-9]*\)(.*)/d'
fi

mv "$tempdir/$logfile" "$(basename "$outputfilename")"-$logfile
mv "$tempdir/$qemuoutputfile" "$(basename "$outputfilename")"-$qemuoutputfile
rm -r "$tempdir"
