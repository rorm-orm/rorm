#!/usr/bin/env python3

import os
import json
import argparse


def setup() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Fix the paths of a SARIF JSON file")
    parser.add_argument("file", help="in file (SARIF JSON)")
    parser.add_argument("--base", help="base path for the source code locations", metavar="dir", required=True)
    return parser


def main(args: argparse.Namespace) -> int:
    with open(args.file) as f:
        data = json.load(f)

    for run in data["runs"]:
        for result in run["results"]:
            for location in result["locations"]:
                p = location["physicalLocation"]["artifactLocation"]["uri"]
                p = os.path.join(args.base, os.path.normpath(p))
                location["physicalLocation"]["artifactLocation"]["uri"] = p

    with open(args.file, "w") as f:
        json.dump(data, f, indent=2)
    return 0


if __name__ == "__main__":
    argv = setup().parse_args()
    exit(main(argv))
