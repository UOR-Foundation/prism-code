Feature: Project admission

  Core and Interaction.applyIntent admit a statement only when its admission
  check allows it, and refuse nine kinds of invalid project.
  Core.gateAdmitsExactly, with addNodeAdmission, addEdgeAdmission,
  markPiiAdmission, requireMemoryAdmission and requirePiiEncryptionAdmission,
  says a statement is admitted exactly when its check admits it. These
  scenarios drive the generated program with the requests the checks refuse
  and hold it to the exact refusal; a request an acceptance vector carries is
  tied to that vector.

  @AD-01 @build
  Scenario: A node whose name is already declared is refused with line N duplicate-node, whatever its kind (acceptance vector 6); Interaction.addNodeAdmission with Core.gateAdmitsExactly says it is admitted exactly when admitNode admits it.
    Given the program generated from the model and scripts that declare a name twice
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N duplicate-node

  @AD-02 @build
  Scenario: An edge with an undeclared endpoint or a pii mark on an undeclared node is refused with line N unknown-node, and existence is checked before the self-edge rule (acceptance vector 5); Interaction.addEdgeAdmission and Interaction.markPiiAdmission with Core.gateAdmitsExactly back both cases.
    Given the program generated from the model and scripts with an unknown source, an unknown target and an unknown pii node
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N unknown-node

  @AD-03 @build
  Scenario: An edge from a node to itself is refused with line N self-edge, ahead of the rules about node kinds (acceptance vector 18); Interaction.addEdgeAdmission with Core.gateAdmitsExactly backs it.
    Given the program generated from the model and scripts with a self-edge on an operator, an output and an input
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N self-edge

  @AD-04 @build
  Scenario: An edge declared twice is refused with line N duplicate-edge, while the reverse edge is a different edge (acceptance vector 19); Interaction.addEdgeAdmission with Core.gateAdmitsExactly backs it.
    Given the program generated from the model and scripts that repeat an edge and that add its reverse
    When each script is checked from standard input
    Then the repeat is refused with exactly line N duplicate-edge and the reverse is accepted

  @AD-05 @build
  Scenario: An edge whose source is an output node is refused with line N output-source, whatever the target kind (acceptance vector 20); Interaction.addEdgeAdmission with Core.gateAdmitsExactly backs it.
    Given the program generated from the model and scripts with an edge from an output to an operator, an output and an input
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N output-source

  @AD-06 @build
  Scenario: An edge whose target is an input node is refused with line N input-target (acceptance vector 21); Interaction.addEdgeAdmission with Core.gateAdmitsExactly backs it.
    Given the program generated from the model and scripts with an edge into an input from an operator and from an input
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N input-target

  @AD-07 @build
  Scenario: A pii mark on a node that is not an input is refused with line N not-an-input (acceptance vector 22); Interaction.markPiiAdmission with Core.gateAdmitsExactly backs it.
    Given the program generated from the model and scripts that mark an operator, an encrypt node and an output as PII
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N not-an-input

  @AD-08 @build
  Scenario: An input marked as PII twice is refused with line N duplicate-pii (acceptance vector 23); Interaction.markPiiAdmission with Core.gateAdmitsExactly backs it.
    Given the program generated from the model and scripts that mark the same input twice, with and without another mark between
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N duplicate-pii

  @AD-09 @build
  Scenario: A second policy memory or a second policy encrypt-pii is refused with line N duplicate-policy, even with the same value or with the other policy between (acceptance vectors 24 and 25); Interaction.requireMemoryAdmission and Interaction.requirePiiEncryptionAdmission with Core.gateAdmitsExactly back it.
    Given the program generated from the model and scripts that repeat each policy
    When each script is checked from standard input
    Then the report is exactly the error outcome and line N duplicate-policy

  @AD-10 @build
  Scenario: The canonical form is the header, the policies with memory before encrypt-pii, the nodes sorted by name, the PII names sorted and the edges sorted by source then target, with names compared by bytes, so the shuffled script with a blank line (acceptance vector 2) and the sorted script (acceptance vector 3) give the same bytes.
    Given the program generated from the model, acceptance vectors 2 and 3 and a shuffled script whose names differ in case and by prefix
    When each script is checked from standard input
    Then the reports are the accepted outcome followed by identical bytes in the sorted order, and vectors 2 and 3 print the same bytes
