#!/usr/bin/env python3

"""
This tool will merge 2 or more results.json files together to present in one "serve" report.

The keys will be merged, so you NEED unique names for EACH engines in EACH json.
e.g. in json1 you could have the engine called "graviton2-tantivy-0.20" and in json2, "graviton3-tantivy-0.20".

Dependencies:
    * mergedeep

Sample usage:
    python3 src/merge_jsons.py --input_jsons foo.json bar.json more.json --output_json ./results2.json
"""
import argparse
import json

from mergedeep import merge

# Installing dependencies:
# pip3 install mergedeep

# sed commands for easy changes to the results.json keys:
# sed -i -e "s/"tantivy-0.20"/"graviton-tantivy-0.20"/g" ./results_graviton.json
# sed -i -e "s/"lucene-9.7.0"/"graviton-lucene-9.7.0"/g" ./results_graviton.json


def parse_args():
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument(
        "-js",
        "--input_jsons",
        dest="input_jsons",
        action="extend",
        nargs="*",
        default=[],
        required=True,
    )

    parser.add_argument(
        "-out",
        "--output_json",
        dest="output_json",
        required=True,
    )
    return parser.parse_args()


if __name__ == "__main__":
    args = parse_args()
    if not args.input_jsons or len(args.input_jsons) < 2 or not args.output_json:
        raise argparse.ArgumentTypeError("You need to specify input json files and an output json file.")

    merged_json = dict()
    for input_json in args.input_jsons:
        with open(input_json, "r") as in_file:
            current_json = json.load(in_file)
            merge(merged_json, current_json)

    with open(args.output_json, "w") as out_file:
        json.dump(merged_json, out_file, indent=2, default=lambda obj: obj.__dict__)
