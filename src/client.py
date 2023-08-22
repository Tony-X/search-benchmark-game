import subprocess
import os
from os import path
import json
import random


COMMANDS = os.environ['COMMANDS'].split(' ')

WARMUP_ITER = int(os.environ['WARMUP_ITER'])
NUM_ITER = int(os.environ['NUM_ITER'])

class SearchClient:

    def __init__(self, engine):
        self.engine = engine
        dirname = os.path.split(os.path.abspath(__file__))[0]
        dirname = path.dirname(dirname)
        dirname = path.join(dirname, "engines")
        cwd = path.join(dirname, engine)
        print(cwd)
        self.process = subprocess.Popen(["make", "--no-print-directory", "serve"],
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            stdin=subprocess.PIPE,
            bufsize=0, # no buffering
            )

    def run_command(self, command):
        self.process.stdin.write(command.encode("utf-8"))
        self.process.stdin.flush()
        return self.process.stdout.readline().strip().decode("utf-8")

    def query(self, query, command):
        query_line = "%s\t%s\n" % (command, query)
        return self.run_command(query_line)

    def get_count(self, query, command):
        # print(query_line)
        query_line = "%s\t%s\n" % (command, query)
        recv = self.run_command(query_line)
        # print('  recv: ' + recv)
        tup = recv.split(' ', 1)

        # Ignore responses from jfr diagnostics, sometimes the length of jfr response tuple is 1
        if 1 <= len(tup) <= 2 and isinstance(tup[0], str) and tup[0][0] == '[':
            elapsed_secs, result, jfr_debug = tup[0].strip('][').split('][')
            # in form of 0.357s, removing trailing 's'
            elapsed_micros = float(elapsed_secs[:-1]) * 1000000
            return elapsed_micros, None

        if len(tup) != 2:
            raise RuntimeError(f'malformed response "{recv}"\n{self.process.stderr.read().decode("utf-8")}')

        elapsed_nanos, result = tup
        elapsed_micros = int(elapsed_nanos) // 1000
        if result == "UNSUPPORTED":
            return elapsed_micros, None
        elif result == '':
            return elapsed_micros, 0

        cnt = int(result)

        return elapsed_micros, cnt

    def close(self):
        self.process.stdin.close()
        self.process.stdout.close()

def drive(queries, client, command):
    for query in queries:
        elapsed_micros, count = client.get_count(query.query, command)
        yield (query, count, elapsed_micros)

class Query(object):
    def __init__(self, query, tags):
        self.query = query
        self.tags = tags

def read_queries(query_path):
    for q in open(query_path):
        c = json.loads(q)
        yield Query(c["query"], c["tags"])


if __name__ == "__main__":
    import sys
    random.seed(2)
    query_path = sys.argv[1]
    engines = sys.argv[2:]
    queries = list(read_queries(query_path))
    results = {}
    for command in COMMANDS:
        results_commands = {}
        for engine in engines:
            engine_results = []
            query_idx = {}
            for query in queries:
                query_result = {
                    "query": query.query,
                    "tags": query.tags,
                    "count": 0,
                    "duration": []
                }
                query_idx[query.query] = query_result
                engine_results.append(query_result)
            print("======================")
            print("BENCHMARKING %s %s" % (engine, command))
            search_client = SearchClient(engine)
            print("--- Warming up ...")
            queries_shuffled = list(queries[:])
            random.seed(2)
            random.shuffle(queries_shuffled)
            for i in range(WARMUP_ITER):
                print("- Warm up Run #%s of %s" % (i + 1, WARMUP_ITER))
                for _ in drive(queries_shuffled, search_client, command):
                    pass
            for i in range(NUM_ITER):
                print("- Run #%s of %s" % (i + 1, NUM_ITER))
                for (query, count, duration) in drive(queries_shuffled, search_client, command):
                    if count is None:
                        query_idx[query.query] = {count: -1, duration: []}
                    else:
                        query_idx[query.query]["count"] = count
                        query_idx[query.query]["duration"].append(duration)
            for query in engine_results:
                query["duration"].sort()
            results_commands[engine] = engine_results
            search_client.close()
        print(results_commands.keys())
        results[command] = results_commands
    with open("results.json" , "w") as f:
        json.dump(results, f, indent=2, default=lambda obj: obj.__dict__)
