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

func writeLine(outputFile *os.File, text string, indentationLevel int) {
	outputFile.WriteString(fmt.Sprintf("%s%s\n", strings.Repeat("    ", indentationLevel), text))
}

func writeTest(outputFile *os.File, fileInfo *fs.FileInfo, indentationLevel int) {
	name := (*fileInfo).Name()
	fmt.Println(name)

	outputFile.WriteString("\n")
	writeLine(outputFile, "#[test]", indentationLevel)
	writeLine(outputFile, fmt.Sprintf("fn %s() -> VMResult {", name), indentationLevel)

	writeLine(outputFile, "}", indentationLevel)
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
		// fmt.Printf("%s, isDir: %v\n", fileInfo.Name(), fileInfo.IsDir())
		name := fileInfo.Name()

		if !fileInfo.IsDir() {
			// If it is a file, write the test in the top level module.
			writeTest(f, &fileInfo, 1)
			continue
		}

		// If it is a directory, create a new test module for its tests.
		if name == "benchmark" {
			// Exclude "benchmark" directory.
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
	// scanner := bufio.NewScanner(f)
	// posts := parsePosts(scanner)
	writeToFile(files)
}
