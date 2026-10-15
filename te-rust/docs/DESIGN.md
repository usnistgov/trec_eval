--------------------------------------------------
Title: te-rust Design Document

Overview:
This document outlines the design for a Rust reimplementation of trec_eval (te-rust). It preserves the original tool’s purpose—evaluating information retrieval runs using standard TREC formats—while replacing legacy C-isms with Rust’s modern, safe abstractions. The goal is to achieve a modular design that separates concerns (CLI, file I/O/parsing, data modeling, metrics calculation, and error handling) without compromising performance or compatibility.

Architectural Goals:
• Modularization: Decompose functionality into clearly defined components.
• Safety & Performance: Leverage Rust’s strong type system and zero-cost abstractions while eliminating manual memory management.
• Backwards Compatibility: Maintain support for the traditional TREC six‐field run and four‐field qrels formats so that existing benchmarking workflows remain valid.
• Testability: Design each module (especially for I/O and parsing) to be unit-testable and easily integrated into an end-to-end test suite.

Module Breakdown:
1. CLI Module
   - Implements argument parsing (using libraries such as clap) to support the established trec_eval interface.
   - Provides configuration options for choosing between traditional and extended input formats.

2. I/O/Parsing Modules
   a) Run File Parser:
      - Supports the traditional six-field TREC run file format.
      - Uses Rust’s safe I/O libraries (e.g., std::fs, BufReader) to read files robustly.
      - Maps each line into a structured RunRecord that captures query ID, topic ID, document ID, rank, score, and any additional fields.
   b) Qrels File Parser:
      - Processes the four-field TREC qrels format.
      - Converts each line into a QrelsRecord that defines topic id, run id, document ID, and relevance judgment.
   - Both parsers include rigorous error handling to report malformed inputs clearly.

3. Data Models
   - Define Rust structs or enums (e.g., RunRecord, QrelsRecord) to represent the parsed data.
   - These models enforce invariants at compile time, ensuring that only valid data flows through the system.

4. Metrics Engine
   - Contains functions that compute evaluation metrics (such as precision, recall, and mean average precision) using parsed data.
   - Is designed to be extended with additional metrics as needed and thoroughly unit-tested.

5. Error Handling
   - Every module uses Rust’s Result and Option types for clear, idiomatic error management.
   - Diagnostic messages report file names, line numbers, and context to aid debugging.

6. Testing Strategy
   - Unit tests for each parser module verify both valid and error-prone scenarios.
   - Integration tests use sample run and qrels files to validate the complete evaluation pipeline.
   - A CI system is planned to run these tests automatically upon changes.

I/O Considerations:
• The design emphasizes safe and efficient file I/O by employing buffered readers and proper error propagation.
• While the current focus is on supporting traditional TREC formats, the design allows for future extension to other input formats through separate parser modules.
• Both synchronous and asynchronous I/O strategies are considered where appropriate to optimize performance without sacrificing clarity.

Backwards Compatibility & Future Extensions:
• The file parsers and metric computations are designed to ensure that the outputs of te-rust match those from trec_eval wherever possible.
• Future work may include adding a Python API wrapper, additional file formats, or more sophisticated ranking metrics, all while keeping the design modular and testable.

Conclusion:
This DESIGN.md document captures a comprehensive plan for reimagining trec_eval in Rust. It prioritizes safety, modularity, and compatibility while outlining clear responsibilities for I/O processing, data modeling, and metric computation. With thorough unit and integration tests in place, te-rust will provide a robust alternative to the legacy C implementation without sacrificing performance or usability.
--------------------------------------------------