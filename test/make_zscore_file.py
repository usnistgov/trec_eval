#!/usr/bin/env python3

from statistics import mean, stdev
import argparse
import collections
import fileinput

ap = argparse.ArgumentParser(
    description='Compute means and standard deviations for a set of input runs, and output a zscores_file for trec_eval')

ap.add_argument('-t', '--topics',
                help='Comma-separated list of topics to include (default: all)')

ap.add_argument('runs',
                nargs='+',
                help='Runs to use for computing measures')

args = ap.parse_args()
if args.topics is not None:
    have_keep_topics = True
    keep_topics = set(args.topics.split(','))
else:
    have_keep_topics = False

scores = collections.defaultdict(dict)

# from https://stackoverflow.com/questions/354038/how-do-i-check-if-a-string-represents-a-number-float-or-int
def is_number(s):
    """ Returns True if string is a number. """
    try:
        float(s)
        return True
    except ValueError:
        return False

for line in fileinput.input(args.runs, encoding='utf-8'):
    measure, topic, score = line.split()
    if topic == 'all':
        continue
    if have_keep_topics and topic not in keep_topics:
        continue
    if is_number(score):
        score = float(score)
    else:
        continue
    if not measure in scores[topic]:
        scores[topic][measure] = []
    scores[topic][measure].append(score)

for topic in scores:
    for measure in scores[topic]:
        mu = mean(scores[topic][measure])
        s2 = stdev(scores[topic][measure])
        print(topic, measure, f'{mu:6.4f}', f'{s2:6.4f}')
