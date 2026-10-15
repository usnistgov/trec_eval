Guidance for AI coding agents, on initiating the trec_eval-in-Rust project.

## Project overview

The goal of this project is to design a clean-room reimplementation of trec_eval in Rust. Moving to a modern language will make maintenance easier and afford new capabilities, like exposing a Python API.

- **trec_eval**: The code for trec_eval is in the directory trec_eval. This is a checkout of the current codebase for trec_eval. There should not be any changes made in this directory.
- **te-rust**: This is the directory for the new trec_eval-in-Rust project.

The new tool will be compatible with existing trec_eval usage:
- The same command-line arguments, with the same semantics
- (There may be new arguments, but they can't conflict with existing ones)
- Accept files in the same formats
- The same measures

Scores obtained from te-rust should be identical to results from trec_eval. When differences exist they must be attributable to numerical differences.

## Workflow

Each stage begins with a design discussion, and creates a design document before any code may be written.

Stages and features are coded in branches, and folded back into the main branch once they have been tested.

## Testing

All components will have unit tests. For measures, these tests must check for corner cases in the measure calculation, such as having no relevant documents for a topic. For files, these tests should check for formatting errors.

For metrics, there should be a common set of test cases that incorporate common edge cases:
- no documents retrieved (an empty ranking)
- no relevant documents retrieved
- no judged documents retrieved (i.e., no documents in the ranking are present in the qrels file)
- A single relevant or nonrelevant document retrieved.
- Cases designed to elicit numerical errors like rounding, infinity, or not-a-number results.

Then individual metrics may add cases that are specific to that metric.

## AI Agent Behavior

You are most helpful when you propose options, describe pros and cons, challenge my preconceived notions, and make constructive suggestions. It is not helpful to tell me that I am brilliant, or otherwise stroke my ego, or describe features or code as beautiful. I am not interested in your opinion. I am interested in your ideas.
