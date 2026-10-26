Feature: Policy compliance

  Compliance.findings computes two kinds of finding, a memory finding and a
  plaintext-PII finding, and Engine.certify accepts a project exactly when
  there are none. Compliance.encryptedPipelineCompliant,
  bypassedPipelineDetected and oversizedPipelineDetected prove the examples,
  and Engine.certifyRequiresCompliance, completedRunAcceptedOnlyIfCompliant
  and failedRunNeverAccepted prove the acceptance guarantee. These scenarios
  drive the generated program and compare the printed reports with the exact
  findings.

  @CP-01 @build
  Scenario: A node whose memory exceeds the policy memory limit is reported as finding memory NODE USED exceeds LIMIT and a node at the limit is not (acceptance vector 4); Compliance.oversizedPipelineDetected proves an oversized pipeline is not compliant.
    Given the program generated from the model and scripts with nodes over and at the memory limit
    When each script is checked from standard input
    Then the report is the rejected outcome with one finding memory line for each node over the limit, and the script at the limit is accepted

  @CP-02 @build
  Scenario: PII that reaches an output node without passing an encrypt node is reported as finding plaintext-pii NODE, and a project where PII reaches two outputs, one through encrypt and one not, reports exactly one such finding (acceptance vector 30); Compliance.bypassedPipelineDetected proves a bypass is detected and Compliance.encryptedPipelineCompliant proves the encrypted pipeline is compliant.
    Given the program generated from the model, scripts with PII reaching an output directly and through operators, acceptance vector 1 and acceptance vector 30
    When each script is checked from standard input
    Then the report names exactly the outputs reached without an encrypt node, vector 1 is accepted and vector 30 has one finding plaintext-pii line, for the output that bypasses encryption

  @CP-03 @build
  Scenario: Findings are printed with every memory finding before every plaintext-pii finding, and each kind in order of node name bytes, whatever the order of the statements (acceptance vector 4); Compliance.findings builds the list and Compliance.sortFindings orders it.
    Given the program generated from the model and a script that declares its nodes and edges in reverse name order
    When the script is checked from standard input
    Then the findings are the memory findings by name and then the plaintext-pii findings by name, before the canonical form

  @CP-04 @build
  Scenario: A project is accepted only if it has no finding, a project with a finding is rejected, and a run that fails on a line is an error and never accepted; Engine.certifyRequiresCompliance, Engine.completedRunAcceptedOnlyIfCompliant and Engine.failedRunNeverAccepted prove it.
    Given the program generated from the model, every acceptance vector within the request maximum and scripts that break a policy and then fail
    When each vector and script is checked from standard input
    Then an accepted report has no finding line, a rejected report has at least one, an error report has none, and a script that fails after breaking a policy is the error

  @CP-05 @build
  Scenario: A policy that the script does not declare produces no finding: no memory policy means no memory finding and no encrypt-pii policy means no plaintext-pii finding, so the project is accepted; Engine.certifyRequiresCompliance says an empty list of findings is accepted.
    Given the program generated from the model and scripts with a node of the largest memory and PII reaching an output, each missing one or both policies
    When each script is checked from standard input
    Then the report is the accepted outcome with no finding line
