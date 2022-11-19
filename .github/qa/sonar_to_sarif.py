#!/usr/bin/env python3

import os
import sys
import json
import argparse
import subprocess


SCHEMA_URL = "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json"


def to_level(severity: str) -> str:
    return {"MINOR": "note", "MAJOR": "warning"}.get(severity, "warning")


def setup() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Convert sonarQubeGenericIssueData JSON file into SARIF JSON file")
    parser.add_argument("-i", "--in-file", help="in file (sonarQubeGenericIssueData JSON)", metavar="f", required=True)
    parser.add_argument("-o", "--out-file", help="out file (SARIF JSON)", metavar="f", required=True)
    parser.add_argument("-f", "--force", action="store_true", help="allow overwriting files")
    parser.add_argument("--base", help="base path for the source code locations", metavar="dir")
    parser.add_argument("--git", help="git repository to add extra revision info to SARIF output", metavar="repo")
    return parser


def main(args: argparse.Namespace) -> int:
    if os.path.exists(args.out_file) and not args.force:
        print(f"Output file {args.out_file!r} already exists. Use '-f' to overwrite it.", file=sys.stderr)
        return 1
    with open(args.in_file) as f:
        content = json.load(f)

    results = []
    counter = 0
    for issue in content["issues"]:
        p = os.path.normpath(issue["primaryLocation"]["filePath"])
        if args.base:
            p = os.path.join(args.base, p)
        results.append({
            "ruleId": issue["ruleId"],
            "level": to_level(issue["severity"]),
            "message": {
                "text": issue["primaryLocation"]["message"]
            },
            "locations": [{
                "id": counter,
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": p
                    },
                    "region": {
                        "startLine": issue["primaryLocation"]["textRange"]["startLine"]
                    }
                }
            }]
        })
        counter += 1

    scanner_version = subprocess.run(
        ["dub", "run", "dscanner", "--", "--version"],
        capture_output=True
    ).stdout.strip().decode("UTF-8").split("\n")[-1]
    run = {
        "tool": {
            "driver": {
                 "name": "dscanner",
                 "version": scanner_version,
                 "semanticVersion": scanner_version[1:]
             }
        },
        "conversion": {
            "tool": {
                "driver": {
                    "name": "sonar2sarif.py"
                }
            },
            "invocation": {
                "commandLine": "python3 " + " ".join(sys.argv),
                "executionSuccessful": True
            }
        },
        "results": results
    }
    if args.git:
        commit_sha = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True).stdout.strip()
        branch = subprocess.run(["git", "rev-parse", "--abbrev-ref", "HEAD"], capture_output=True).stdout.strip()
        version_info = {
            "repositoryUri": args.git,
            "revisionId": commit_sha.decode("UTF-8"),
            "branch": branch.decode("UTF-8")
        }
        run["versionControlProvenance"] = [version_info]

    data = {"version": "2.1.0", "runs": [run], "$schema": SCHEMA_URL}
    with open(args.out_file, "w") as f:
        json.dump(data, f, indent=2)
    return 0


if __name__ == "__main__":
    argv = setup().parse_args()
    exit(main(argv))
