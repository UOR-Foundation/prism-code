Feature: Generated command line

  The prism-code command line is generated from the application model, so
  these scenarios drive the generated program and compare it with what the
  model declares. Each scenario discharges one registered ID.

  @CL-01 @build
  Scenario: prism-code check FILE, check - and check with the request on standard input print the same bytes for every acceptance vector within the request maximum, and those bytes equal the vector's modeled response.
    Given the program generated from the model and every acceptance vector within the request maximum
    When the vector is checked from a file, from - and from standard input
    Then the three standard outputs are identical, equal the modeled response and standard error is empty

  @CL-02 @build
  Scenario: The exit status of prism-code check is the model's code for the outcome the report begins with: 0 for status accepted, 1 for status rejected and 2 for status error.
    Given the program generated from the model and the model's exit-code map
    When every acceptance vector within the request maximum is checked
    Then the exit status is the code of the outcome whose prefix begins the report, and each of the three outcomes occurs

  @CL-03 @build
  Scenario: A request longer than the request maximum is refused before dispatch: standard output stays empty, standard error names InputLimit and the exit status is the code of the error outcome.
    Given the program generated from the model and every acceptance vector over the request maximum
    When the vector is checked from a file and from standard input
    Then standard output is empty, standard error begins with the InputLimit diagnostic and the exit status is the error outcome's code

  @CL-04 @build
  Scenario: No Rust source in the repository implements dispatch, request parsing or report formatting; the command line is generated from the model.
    Given every Rust source file under crates and xtask
    When the files are scanned for the model's report prefixes, the request header and the dispatch entry
    Then no file names one, and a planted sample that does is reported

  @CL-05 @build
  Scenario: --help prints the model's purpose, statement reference, report format, status texts, recovery text for every error and finding code, example names and exit-code table, and exits 0.
    Given the program generated from the model
    When prism-code --help runs
    Then standard output holds each modeled section, the purpose, every example name and a row for every exit code, and the exit status is 0

  @CL-06 @build
  Scenario: --version prints the model's cargoName and cargoVersion on one line, prism-code 0.1.0, and exits 0.
    Given the program generated from the model
    When prism-code --version runs
    Then standard output is the cargoName, a space and the cargoVersion followed by a newline, the version equals the system model's product version, and the exit status is 0

  @CL-07 @build
  Scenario: examples lists the model's example names, examples NAME prints that script unchanged, and piping each script to check gives the report and exit status of the acceptance vector it is drawn from.
    Given the program generated from the model and the model's named examples
    When examples runs without a name, with each name, and each script is piped to check
    Then the listing is the names in order, each script equals the model text, and the checked report equals the vector's modeled response with its outcome's exit status

  @CL-08 @build
  Scenario: Every guidance string, example and code explanation in the workbench model also appears in the --help or examples output.
    Given the workbench's statement forms, status texts, recovery texts and the modeled examples
    When the output of --help and examples NAME is searched for each
    Then every statement form, status text, code with its explanation and fix, and example script is present

  @CL-09 @build
  Scenario: An unknown flag is refused with the UnknownFlag diagnostic on standard error, the model's usage code and empty standard output.
    Given the program generated from the model
    When prism-code is run with a flag it does not define
    Then standard error begins with the UnknownFlag diagnostic and the exit status is the usage code

  @CL-10 @build
  Scenario: A second operand is refused with the ExtraArgument diagnostic on standard error, the model's usage code and empty standard output.
    Given the program generated from the model
    When prism-code check is run with two operands
    Then standard error begins with the ExtraArgument diagnostic and the exit status is the usage code

  @CL-11 @build
  Scenario: Running without a command is refused with the MissingCommand diagnostic on standard error, the model's usage code and empty standard output.
    Given the program generated from the model
    When prism-code is run with no arguments
    Then standard error begins with the MissingCommand diagnostic and the exit status is the usage code

  @CL-12 @build
  Scenario: An unknown command is refused with the UnknownCommand diagnostic on standard error, the model's usage code and empty standard output.
    Given the program generated from the model
    When prism-code is run with a command it does not define
    Then standard error begins with the UnknownCommand diagnostic and the exit status is the usage code

  @CL-13 @build
  Scenario: An example name the model does not declare is refused with the UnknownExample diagnostic on standard error, the model's usage code and empty standard output.
    Given the program generated from the model
    When prism-code examples is run with a name that is not an example
    Then standard error begins with the UnknownExample diagnostic and the exit status is the usage code

  @CL-14 @build
  Scenario: A request path that does not exist is refused with the InputMissing diagnostic on standard error, the model's I/O code and empty standard output.
    Given the program generated from the model
    When prism-code check is run on a path that does not exist
    Then standard error begins with the InputMissing diagnostic and the exit status is the I/O code

  @CL-15 @build
  Scenario: A request path that cannot be opened, here a symbolic link, is refused with the InputOpen diagnostic on standard error, the model's I/O code and empty standard output.
    Given the program generated from the model
    When prism-code check is run on a symbolic link to a valid script
    Then standard error begins with the InputOpen diagnostic and the exit status is the I/O code

  @CL-16 @build
  Scenario: A request path that is not a regular file is refused with the InputType diagnostic on standard error, the model's I/O code and empty standard output.
    Given the program generated from the model
    When prism-code check is run on a directory
    Then standard error begins with the InputType diagnostic and the exit status is the I/O code

  @CL-17 @build
  Scenario: A request that fails while it is being read is refused with the InputRead diagnostic on standard error, the model's I/O code and empty standard output.
    Given the program generated from the model
    When prism-code check reads standard input that is a directory
    Then standard error begins with the InputRead diagnostic and the exit status is the I/O code

  @CL-18 @build
  Scenario: A report that cannot be written to standard output is refused with the OutputWrite diagnostic on standard error and the model's I/O code.
    Given the program generated from the model
    When prism-code check writes its report to a full device
    Then standard error begins with the OutputWrite diagnostic and the exit status is the I/O code
