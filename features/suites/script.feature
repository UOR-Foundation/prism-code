Feature: Script parsing

  The script parser in Interaction turns each line into a statement or a typed
  parse error. These scenarios drive the generated program with scripts that
  use each statement form and each parse error, and compare the printed bytes
  with the exact report. A request an acceptance vector carries is tied to
  that vector.

  @SC-01 @build
  Scenario: Interaction.parseNodeFields reads node NAME KIND MEMORY for each of the kinds input, operator, encrypt and output, and the generated program accepts a script of such a statement and prints it back as its canonical form.
    Given the program generated from the model and one script for each node kind
    When each script is checked from standard input
    Then the report is the accepted outcome followed by the header and the node line unchanged, with the accepted exit status and empty standard error

  @SC-02 @build
  Scenario: Interaction.parseEdgeFields reads edge SOURCE TARGET, and the generated program accepts a script that connects two declared nodes and prints the edge in its canonical form.
    Given the program generated from the model and scripts that declare two nodes and connect them
    When each script is checked from standard input
    Then the report is the accepted outcome followed by the canonical form that includes the edge line

  @SC-03 @build
  Scenario: Interaction.parsePiiFields reads pii NAME, and the generated program accepts the mark on an input node, prints it in its canonical form and reports no finding while the encrypt-pii policy is absent.
    Given the program generated from the model and scripts that mark an input node as PII, with and without an output it feeds
    When each script is checked from standard input
    Then the report is the accepted outcome with the pii line in the canonical form and no finding line

  @SC-04 @build
  Scenario: Interaction.parsePolicyFields reads policy memory LIMIT, and the generated program accepts it, prints it first among the statements of the canonical form and does not count a node at the limit as over it.
    Given the program generated from the model and scripts with a memory limit equal to a node's memory
    When each script is checked from standard input
    Then the report is the accepted outcome with the policy memory line before the node lines

  @SC-05 @build
  Scenario: Interaction.parsePolicyFields reads policy encrypt-pii with no operand, and the generated program accepts it and prints it in its canonical form.
    Given the program generated from the model and a script that requires PII encryption
    When the script is checked from standard input
    Then the report is the accepted outcome with the policy encrypt-pii line in the canonical form

  @SC-06 @build
  Scenario: A line whose command is not node, edge, pii or policy, or whose policy is not memory or encrypt-pii, is refused with line N unknown-command, where N counts blank lines (acceptance vector 14).
    Given the program generated from the model and scripts with an unknown command, a misspelt command, an unknown policy and a blank line before the bad line
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N unknown-command, with the error exit status

  @SC-07 @build
  Scenario: A statement with too few or too many operands, including a line of more than five fields, is refused with line N wrong-arity (acceptance vector 15).
    Given the program generated from the model and scripts that give each statement form too few and too many operands
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N wrong-arity

  @SC-08 @build
  Scenario: A line with an empty field, from a doubled, leading or trailing space, is refused with line N empty-field before its command is read (acceptance vector 8).
    Given the program generated from the model and scripts with a doubled space, a leading space, a trailing space and a line of one space
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N empty-field

  @SC-09 @build
  Scenario: A node whose kind is not input, operator, encrypt or output is refused with line N unknown-kind, ahead of the memory being read (acceptance vector 7).
    Given the program generated from the model and scripts with a misspelt, capitalised and unknown node kind
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N unknown-kind

  @SC-10 @build
  Scenario: A node memory or memory limit that is not an unsigned 64-bit decimal is refused with line N invalid-number (acceptance vectors 16 and 17).
    Given the program generated from the model and scripts with a letter, a negative number, a fraction and a number over the largest UInt64 as the memory or the limit
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N invalid-number
