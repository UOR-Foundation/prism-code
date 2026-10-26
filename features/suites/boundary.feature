Feature: Request and response boundaries

  Specification.dispatch bounds a request at 4096 bytes and a response at
  16384 bytes. These scenarios hold the generated program to the exact report
  for each boundary, with requests of exactly 4096 and 4097 bytes. A refusal
  that no request within the request maximum can reach, request too-many-lines
  and response too-large, is held to the arithmetic over the modeled limits
  and to the densest request; the model has no theorem for either bound
  (features/suites/README.md).

  @BD-01 @build
  Scenario: An empty request is refused with request empty (acceptance vector 11).
    Given the program generated from the model and an empty request
    When the request is checked from standard input
    Then the report is exactly the error outcome and request empty, with the error exit status

  @BD-02 @build
  Scenario: A request of exactly 4097 bytes is refused as request too-large (acceptance vector 13): the command line refuses it before dispatch with InputLimit, and the verification evidence holds the Core-Wasm entry and the browser view to the modeled report.
    Given the program generated from the model, its verification evidence and requests of exactly 4097 bytes, one that is not a script and one that is the accepted 4096-byte script and a newline
    When each request is checked from standard input
    Then standard output is empty, standard error is the InputLimit diagnostic, the exit status is the limit code and the evidence attests the modeled request too-large report

  @BD-03 @build
  Scenario: A request that is not valid UTF-8 is refused with request invalid-utf8, wherever the invalid byte is (acceptance vector 10).
    Given the program generated from the model and requests with a lone byte, a bad continuation and a bad byte after a valid script
    When each request is checked from standard input
    Then the report is exactly the error outcome and request invalid-utf8

  @BD-04 @build
  Scenario: A request cannot exceed the 4097-line limit of request too-many-lines within the 4096-byte request maximum, because N bytes hold at most N newlines and N + 1 lines, so no acceptance vector produces it: the request of 4096 newlines has 4097 lines and is still split, and one byte more is refused as too large.
    Given the program generated from the model and the modeled line and request maximums
    When a request of 4096 newlines, the longest script and a request of 4097 newlines are checked from standard input
    Then the line maximum is the request maximum plus one, the first two are answered by their own reports and the third is refused with InputLimit

  @BD-05 @build
  Scenario: A request whose first line is not the script header is refused with line 1 missing-header, including a changed version, a trailing space, a different case and a blank first line (acceptance vector 9).
    Given the program generated from the model and requests whose first line is not the header
    When each request is checked from standard input
    Then the report is exactly the error outcome and line 1 missing-header

  @BD-06 @build
  Scenario: A response cannot exceed the 16384-byte limit of response too-large from a request within the 4096-byte maximum, so no acceptance vector produces it: the densest request found, PII from one input to the most outputs over a memory limit of zero, gives a response under the limit.
    Given the program generated from the model and a request of up to 4096 bytes that gives every output a memory finding and a plaintext-pii finding
    When the request is checked from standard input
    Then the report has two findings for every output, is more than twice the request and is shorter than the response maximum

  @BD-07 @build
  Scenario: A request of exactly 4096 bytes is accepted, and so is one byte fewer (acceptance vector 12).
    Given the program generated from the model and the 4096-byte acceptance vector
    When the request is checked from standard input
    Then the report is the accepted outcome with the header, with the accepted exit status
