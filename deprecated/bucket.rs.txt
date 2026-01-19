// This file will search through the graph directory.

// Note: likely need a better naming convention for graph files to make searching faster.

// Note: bucketing search could just happen async or on time cycles so we are doing that much searching. We dont want to do
// immediatley because that would break stuff prob..

// Note: We need a smart way to keep track of graphs once they are bucketed.

// Fields we will use:
/*
{
  "script_name": "test.py",
  "error_type": "NameError",    <-----
  "error_list": "SyntaxError | NameError",   <-----

*/
// These are bigs ones and sit at the top of the graph file. This will make searching and bucketing easy.


// FINAL STEP: combine with graph.rs to make context.rs file