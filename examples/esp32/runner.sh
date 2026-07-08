#!/bin/bash

if [ $# -eq 0 ]; then
  echo "No arguments supplied! Provide path to ELF as argument"
  exit 1
fi

elf=$1

espflash flash "$elf" --monitor