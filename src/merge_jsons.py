#!/usr/bin/env python3

"""
This will merge 2 or more results.json files together to present in one "serve" report.

This tool will replace keys in the merge, so you NEED unique names for EACH engines in EACH json.
e.g. in json1 you could have the engine called "graviton2-tantivy-0.20" and in json2, "graviton3-tantivy-0.20".
Same for other engines.

Sample usage:
    python3 src/merge_jsons.py --input_jsons foo.json bar.json more.json --output_json ./results2.json


Dependencies:
    * mergedeep

pip3 install mergedeep to download this.
"""
import argparse
import json

from mergedeep import merge


def parse_args():
    parser = argparse.ArgumentParser()

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
        json.dump(merged_json, out_file)
