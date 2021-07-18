package main

import (
	"fmt"
	"log"
	"os"
	"strings"
)

const OUTPUT_FILE = "./tests.rs"

func writeLine(f *os.File, text string, indentationLevel int) {
	f.WriteString(fmt.Sprintf("%s%s\n", strings.Repeat("    ", indentationLevel), text))
}

func writeTest(f *os.File, testName string, indentationLevel int) {
	f.WriteString("\n")
	writeLine(f, "#[test]", indentationLevel)
	writeLine(f, fmt.Sprintf("fn %s() -> VMResult {", testName), indentationLevel)
	writeLine(f, "}", indentationLevel)
}

func writeModule(f *os.File, moduleName string, indentationLevel int) {
	f.WriteString("\n")
	writeLine(f, fmt.Sprintf("mod %s {", moduleName), indentationLevel)
	writeLine(f, "use super::*;", indentationLevel+1)
	writeLine(f, "}", indentationLevel)
}

func writePostsToFile() {
	f, err := os.Create(OUTPUT_FILE)
	if err != nil {
		log.Fatal(err)
	}
	defer f.Close()

	writeLine(f, "#[cfg(test)]", 0)
	writeLine(f, "mod tests {", 0)
	writeLine(f, "use super::*;", 1)
	writeLine(f, "use crate::value::Value;", 1)
	writeModule(f, "variable", 1)
	writeLine(f, "}", 0)
}

func main() {
	// scanner := bufio.NewScanner(f)
	// posts := parsePosts(scanner)
	writePostsToFile()
}
