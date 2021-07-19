package main

import (
	"bufio"
	"fmt"
	"io/fs"
	"io/ioutil"
	"log"
	"os"
	"regexp"
	"strings"
)

const OUTPUT_FILE = "./tests.rs"
const INPUT_DIRECTORY = "./test/"

func writeLine(outputFile *os.File, text string, indentationLevel int) {
	outputFile.WriteString(fmt.Sprintf("%s%s\n", strings.Repeat("    ", indentationLevel), text))
}

func writeTest(outputFile *os.File, fileInfo *fs.FileInfo, moduleName string, indentationLevel int) {
	if !strings.HasSuffix((*fileInfo).Name(), ".lox") {
		log.Fatal("Invalid file input. Only .lox files should be present in the input directory.")
	}
	name := strings.Replace((*fileInfo).Name(), ".lox", "", 1)

	outputFile.WriteString("\n")
	writeLine(outputFile, "#[test]", indentationLevel)
	writeLine(outputFile, fmt.Sprintf("fn %s() -> VMResult {", name), indentationLevel)

	// Write test body.
	var path string
	if len(moduleName) > 0 {
		path = INPUT_DIRECTORY + moduleName + "/" + (*fileInfo).Name()
	} else {
		path = INPUT_DIRECTORY + (*fileInfo).Name()
	}
	f, err := os.Open(path)
	if err != nil {
		log.Fatal(err)
	}
	defer f.Close()
	sc := bufio.NewScanner(f)

	writeLine(outputFile, "let source = r#\"", indentationLevel+1)
	assertError := ""
	assertValues := make([]string, 0)
	for sc.Scan() {
		line := sc.Text()
		writeLine(outputFile, line, 0)

		// There may be edge cases, error comment not always consistent?
		matchError, _ := regexp.MatchString("(?i)error", line)
		// There is at least one test file where there are two error comments,
		// the second error is for Java (unexpected_character.lox)
		if matchError && len(assertError) == 0 {
			assertError = strings.SplitAfter(line, ": ")[1]
		}
		matchExpect, _ := regexp.MatchString("// expect: ", line)
		if matchExpect {
			assertValues = append(assertValues, strings.SplitAfter(line, ": ")[1])
		}
	}
	writeLine(outputFile, "\"#", 0)
	writeLine(outputFile, ".to_string();", indentationLevel+1)
	writeLine(outputFile, "let mut vm = VM::init();", indentationLevel+1)

	if len(assertValues) > 0 {
		// This test expects certain values to be printed.
		writeLine(outputFile, "vm.interpret(source)?;", indentationLevel+1)

		// Write one assertion for each expected value.
		for i := len(assertValues) - 1; i >= 0; i-- {
			writeLine(outputFile, "assert_eq!(", indentationLevel+1)
			writeLine(outputFile, fmt.Sprintf("\"%s\".to_string(),", assertValues[i]), indentationLevel+2)
			writeLine(outputFile, "vm.printed_values.pop().unwrap().to_string()", indentationLevel+2)
			writeLine(outputFile, ");", indentationLevel+1)
		}

	} else if len(assertError) > 0 {
		// This test expects a specific error.
		writeLine(outputFile, "vm.interpret(source);", indentationLevel+1)
		writeLine(outputFile, "assert_eq!(", indentationLevel+1)
		writeLine(outputFile, fmt.Sprintf("\"%s\",", assertError), indentationLevel+2)
		writeLine(outputFile, "vm.latest_error_message", indentationLevel+2)
		writeLine(outputFile, ");", indentationLevel+1)
	}

	writeLine(outputFile, "Ok(())", indentationLevel+1)
	writeLine(outputFile, "}", indentationLevel)
}

func writeModule(outputFile *os.File, moduleName string, modFilesInfo []fs.FileInfo, indentationLevel int) {
	outputFile.WriteString("\n")
	writeLine(outputFile, fmt.Sprintf("mod %s {", moduleName), indentationLevel)
	writeLine(outputFile, "use super::*;", indentationLevel+1)

	for _, tf := range modFilesInfo {
		writeTest(outputFile, &tf, moduleName, indentationLevel+1)
	}

	// Closing bracket for the module.
	writeLine(outputFile, "}", indentationLevel)
}

func writeToFile(files []fs.FileInfo) {
	f, err := os.Create(OUTPUT_FILE)
	if err != nil {
		log.Fatal(err)
	}
	defer f.Close()

	// Write the top level tests module.
	writeLine(f, "#[cfg(test)]", 0)
	writeLine(f, "mod tests {", 0)
	writeLine(f, "use super::*;", 1)
	writeLine(f, "use crate::value::Value;", 1)

	for _, fileInfo := range files {
		name := fileInfo.Name()

		if !fileInfo.IsDir() {
			// If it is a file, write the test in the top level module.
			writeTest(f, &fileInfo, "", 1)
			continue
		}

		// If it is a directory, create a new test module for its tests.
		// if name == "benchmark" || name == "regression" {
		if name != "assignment" &&
			name != "block" &&
			name != "bool" &&
			name != "comments" &&
			// name != "expressions" &&
			// name != "operator" &&
			name != "print" &&
			name != "string" {
			// Directories to exclude.
			continue
		}
		modTestFilesInfo, err := ioutil.ReadDir(INPUT_DIRECTORY + name)
		if err != nil {
			log.Fatal(err)
		}
		writeModule(f, name, modTestFilesInfo, 1)
	}

	// Closing bracket for the top level tests module.
	writeLine(f, "}", 0)
}

func main() {
	files, err := ioutil.ReadDir(INPUT_DIRECTORY)
	if err != nil {
		log.Fatal(err)
	}
	writeToFile(files)
}
