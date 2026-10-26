Feature: Web and command-line parity

  The page and the command line call the same entry, so for the same script
  they must produce the same observable result. SDK verification runs every
  acceptance vector in Core-Wasm and in the portable browser view and holds both
  to the modeled response; this scenario holds the generated command line to the
  same response and does not run the page again.

  @PA-01 @build
  Scenario: For every acceptance vector, the response bytes that prismpm verify holds the Core-Wasm entry and the browser view to equal what the generated command line prints, and a vector the command line refuses before or instead of dispatch is compared by that refusal; verification evidence that attests the program under test must exist, and a changed command-line output fails the comparison.
    Given the program generated from the model, the verification evidence prismpm verify published for it and every acceptance vector
    When each vector is checked from standard input, and again through a wrapper that appends a newline to the program's output
    Then each vector within the request maximum prints its modeled response and outcome code, an oversized vector and a workbench frame are refused with empty standard output, and the wrapped program fails the comparison on every vector
