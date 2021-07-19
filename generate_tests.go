package main

import (
	"fmt"
	"io/fs"
	"io/ioutil"
	"log"
	"os"
	"strings"
)

const OUTPUT_FILE = "./tests.rs"

const INPUT_DIRECTORY = "./test/"

// const INPUT_DIRECTORY = "./test/assignment/"

func writeLine(f *os.File, text string, indentationLevel int) {
	f.WriteString(fmt.Sprintf("%s%s\n", strings.Repeat("    ", indentationLevel), text))
}

func writeTest(f *os.File, testName string, indentationLevel int) {
	f.WriteString("\n")
	writeLine(f, "#[test]", indentationLevel)
	writeLine(f, fmt.Sprintf("fn %s() -> VMResult {", testName), indentationLevel)
	writeLine(f, "}", indentationLevel)
}

func writeModule(outputFile *os.File, moduleName string, moduleTestFiles []fs.FileInfo, indentationLevel int) {
	outputFile.WriteString("\n")
	writeLine(outputFile, fmt.Sprintf("mod %s {", moduleName), indentationLevel)
	writeLine(outputFile, "use super::*;", indentationLevel+1)

	for _, tf := range moduleTestFiles {
		fmt.Println(moduleName + "/" + tf.Name())
	}

	// Closing bracket for the module.
	writeLine(outputFile, "}", indentationLevel)
}

func writeToFile() {
	files, err := ioutil.ReadDir(INPUT_DIRECTORY)
	if err != nil {
		log.Fatal(err)
	}

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

	for _, fileOrDir := range files {
		if fileOrDir.IsDir() {
			// If it is a directory, create a new test module for its tests.
			moduleName := fileOrDir.Name()
			// dir, err := os.Open(fileOrDir.Name())
			// if err != nil {
			// 	log.Fatal(err)
			// }
			modTestFiles, err := ioutil.ReadDir(INPUT_DIRECTORY + fileOrDir.Name())
			if err != nil {
				log.Fatal(err)
			}
			writeModule(f, moduleName, modTestFiles, 1)
		}
	}

	// Closing bracket for the top level tests module.
	writeLine(f, "}", 0)
}

func main() {
	// scanner := bufio.NewScanner(f)
	// posts := parsePosts(scanner)
	writeToFile()
}
