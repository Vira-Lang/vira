package main

import (
	"flag"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
)

const repoURL = "https://bytes.io/packages/"

func downloadPackage(pkgName string, destDir string) error {
	url := repoURL + pkgName + ".tar.gz"
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("failed to download: %s", resp.Status)
	}

	filePath := filepath.Join(destDir, pkgName+".tar.gz")
	file, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer file.Close()

	_, err = io.Copy(file, resp.Body)
	return err
}

func install(pkgName string, inProject bool) error {
	var destDir string
	if inProject {
		destDir = filepath.Join("build", "dependencies")
		os.MkdirAll(destDir, 0755)
	} else {
		destDir = os.Getenv("HOME") + "/.vira/libs"
	}
	return downloadPackage(pkgName, destDir)
}

func remove(pkgName string) error {
	// Stub: remove from libs
	path := os.Getenv("HOME") + "/.vira/libs/" + pkgName + ".tar.gz"
	return os.Remove(path)
}

func update() error {
	// Stub: update all
	fmt.Println("Updating all packages...")
	return nil
}

func upgrade() error {
	// Stub: upgrade binaries
	fmt.Println("Upgrading Vira...")
	return nil
}

func refresh() error {
	// Stub: refresh cache
	fmt.Println("Refreshing repo...")
	return nil
}

func search(query string) error {
	// Stub: search
	fmt.Printf("Search results for %s:\n- math\n- io\n", query)
	return nil
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("Usage: vira-packages <command> [args]")
		fmt.Println("Commands: install, remove, update, upgrade, refresh, search")
		os.Exit(1)
	}

	command := os.Args[1]
	args := os.Args[2:]

	switch command {
	case "install":
		inProject := flag.Bool("in-project", false, "Install in project")
		flag.CommandLine.Parse(args)
		pkgName := flag.Arg(0)
		if pkgName == "" {
			fmt.Println("Provide package name")
			os.Exit(1)
		}
		err := install(pkgName, *inProject)
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		fmt.Println("Installed", pkgName)
	case "remove":
		if len(args) < 1 {
			fmt.Println("Provide package name")
			os.Exit(1)
		}
		err := remove(args[0])
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
		fmt.Println("Removed", args[0])
	case "update":
		err := update()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
	case "upgrade":
		err := upgrade()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
	case "refresh":
		err := refresh()
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
	case "search":
		if len(args) < 1 {
			fmt.Println("Provide query")
			os.Exit(1)
		}
		err := search(args[0])
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
	default:
		fmt.Println("Unknown command")
		os.Exit(1)
	}
}
