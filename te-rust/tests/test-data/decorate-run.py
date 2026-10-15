#!/usr/bin/env python3

import argparse
import collections
import subprocess

ap = argparse.ArgumentParser(
    description='Given a run and a qrels, print topic, docid, score and rel info from the run')

ap.add_argument('qrels_file')
ap.add_argument('run_file')

args = ap.parse_args()

qrels = collections.defaultdict(dict)
with open(args.qrels_file, 'r') as qrels_file:
    for line in qrels_file:
        topic, _, docid, rel = line.strip().split()
        qrels[topic][docid] = rel

teval = subprocess.run(['sort', '-k1,1', '-k5,5nr', '-k3,3r',
                        args.run_file],
                       text=True,
                       stdout=subprocess.PIPE)

cur_topic = None
for line in teval.stdout.split('\n'):
    if len(line) == 0:
        continue
    topic, _, docid, _, sim, runtag = line.split()
    if topic != cur_topic:
        cur_topic = topic
        rank = 0
        rel_ret = 0
        num_rel = 0
        for d, rel in qrels[topic].items():
            if int(rel) > 0:
                num_rel += 1
        p = 0.0
        r = 0.0

    rank += 1

    if docid in qrels[topic]:
        rel = qrels[topic][docid]
        if int(rel) > 0:
            rel_ret += 1
            p = float(rel_ret) / rank
            r = float(rel_ret) / num_rel
    else:
        rel = '-'

    print(topic, rel, docid, sim, end='')
    if not rel.startswith('-') and int(rel) > 0:
        print(' ', f'p={p:.4}', f'r={r:.4}')
    else:
        print()
