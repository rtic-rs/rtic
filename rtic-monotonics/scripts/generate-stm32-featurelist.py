#!/usr/bin/env python3

import urllib.request
import json
from argparse import ArgumentParser


def main():
    parser = ArgumentParser()
    parser.add_argument("stm32_metapac_version")
    args = parser.parse_args()

    with urllib.request.urlopen('https://crates.io/api/v1/crates/stm32-metapac/' + args.stm32_metapac_version) as f:
        for name in sorted(json.loads(f.read().decode('utf-8'))["version"]["features"].keys()):
            if name.startswith("stm32"):
                print(f'{name} = ["dep:cortex-m", "stm32-metapac/{name}"]')


if __name__ == "__main__":
    main()
