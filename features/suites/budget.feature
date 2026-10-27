Feature: Quality budgets

  The quality model gives the Core-Wasm guest, the browser closure and the
  command-line check a threshold each (src/Quality.lex.tex). These scenarios
  measure what prismpm build published for the program under test, report each
  measured value beside its budget and hold it to the threshold. The measured
  values are open claims; only the comparison with the budget is asserted.

  @QB-01 @build
  Scenario: The Core-Wasm guest prismpm build publishes is no larger than the byte threshold of measureGuestSize in the quality model, and a guest one byte over that threshold is refused; the measured size is reported beside the budget and is not itself a claim.
    Given the build that published the program under test and the measureGuestSize threshold
    When the size of the one guest under core-wasm is measured, and a size one byte over the threshold is compared
    Then the measured size is reported and is within the threshold, and the larger size is refused

  @QB-02 @build
  Scenario: The six-file browser closure prismpm build publishes is no larger in total than the byte threshold of measureClosureSize in the quality model; the measured size is reported beside the budget and is not itself a claim.
    Given the build that published the program under test and the measureClosureSize threshold
    When the six files under view/browser are measured
    Then their total size is reported and is within the threshold

  @QB-03 @build
  Scenario: The median of 11 runs of dataflow-policy check on acceptance vector 12, the 4096-byte request, process start included and each run giving the modeled response, is within the millisecond threshold of measureCliLatency in the quality model; the measured time is reported beside the budget and is not itself a claim.
    Given the program generated from the model, acceptance vector 12 and the measureCliLatency threshold
    When vector 12 is checked 11 times from standard input
    Then every run prints the modeled response, and the median time is reported and is within the threshold
