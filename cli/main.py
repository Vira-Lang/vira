# Further Expanded Python CLI for Vira
# Added more features: configuration management, better project handling, dependency resolution from bytes.yml, verbose mode, etc.

import os
import sys
import subprocess
import shutil
import yaml
import logging
import argparse
from pathlib import Path
from typing import List, Dict, Optional
from rich.console import Console
from rich.panel import Panel
from rich.prompt import Prompt, Confirm
from rich.table import Table
from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn
from rich.text import Text
from rich import print as rprint

console = Console()

VIRA_HOME = Path.home() / ".vira"
VIRA_BIN = VIRA_HOME / "bin"
VIRA_LIBS = VIRA_HOME / "libs"
VIRA_LOGS = VIRA_HOME / "logs"
VIRA_CACHE = VIRA_HOME / "cache"
VIRA_CONFIG = VIRA_HOME / "config.yml"

VIRA_HOME.mkdir(parents=True, exist_ok=True)
VIRA_BIN.mkdir(parents=True, exist_ok=True)
VIRA_LIBS.mkdir(parents=True, exist_ok=True)
VIRA_LOGS.mkdir(parents=True, exist_ok=True)
VIRA_CACHE.mkdir(parents=True, exist_ok=True)

# Setup logging with levels
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[
        logging.FileHandler(VIRA_LOGS / "vira.log"),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger("vira")

def run_subprocess(cmd: List[str], capture_output: bool = False, timeout: Optional[int] = None) -> str:
    try:
        if capture_output:
            output = subprocess.check_output(cmd, stderr=subprocess.STDOUT, timeout=timeout).decode()
            logger.info(f"Command output: {output}")
            return output
        else:
            subprocess.check_call(cmd, timeout=timeout)
            logger.info(f"Command {cmd} executed successfully")
    except subprocess.CalledProcessError as e:
        error_msg = e.output.decode() if e.output else str(e)
        console.print(f"[bold red]Error executing command:[/bold red] {error_msg}")
        logger.error(f"Error running {cmd}: {error_msg}")
        sys.exit(1)
    except subprocess.TimeoutExpired:
        console.print("[bold red]Command timed out.[/bold red]")
        logger.error(f"Timeout for {cmd}")
        sys.exit(1)
    return ""

def get_platform() -> str:
    platforms = {
        "linux": "linux",
        "win32": "windows",
        "darwin": "macos"
    }
    return platforms.get(sys.platform, "unknown")

def find_bytes_yml(start_dir: Path = Path.cwd()) -> Optional[Path]:
    current = start_dir
    while current != current.parent:
        bytes_path = current / "bytes.yml"
        if bytes_path.exists():
            return bytes_path
        current = current.parent
    return None

def load_bytes_yml(path: Optional[Path] = None) -> Dict:
    if path is None:
        path = find_bytes_yml()
    if path:
        try:
            with open(path, "r") as f:
                config = yaml.safe_load(f)
                logger.info(f"Loaded config from {path}")
                return config or {}
        except yaml.YAMLError as e:
            console.print(f"[bold red]Invalid YAML in {path}:[/bold red] {e}")
            logger.error(f"YAML error in {path}: {e}")
            sys.exit(1)
    console.print("[bold yellow]No bytes.yml found. Using defaults.[/bold yellow]")
    return {}

def get_source_dir(config: Dict) -> str:
    return config.get("<>", "cmd") or "cmd"

def save_vira_config(config: Dict):
    with open(VIRA_CONFIG, "w") as f:
        yaml.dump(config, f)
    logger.info("Saved Vira config")

def load_vira_config() -> Dict:
    if VIRA_CONFIG.exists():
        with open(VIRA_CONFIG, "r") as f:
            return yaml.safe_load(f) or {}
    default_config = {"version": "0.1.0", "verbose": False}
    save_vira_config(default_config)
    return default_config

def resolve_dependencies(config: Dict):
    deps = config.get("dependencies", {})
    for dep, version in deps.items():
        # Check if installed
        dep_path = VIRA_LIBS / f"{dep}-{version}"
        if not dep_path.exists():
            console.print(f"[bold yellow]Installing missing dependency: {dep}@{version}[/bold yellow]")
            run_subprocess([str(VIRA_BIN / "vira-packages"), "install", f"{dep}@{version}"])

class ViraCLI:
    def __init__(self):
        self.config = load_vira_config()
        if self.config.get("verbose"):
            logger.setLevel(logging.DEBUG)
        self.parser = argparse.ArgumentParser(description="Vira CLI", add_help=False)
        self.parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose mode")
        self.parser.add_argument("-h", "--help", action="store_true", help="Show help")
        subparsers = self.parser.add_subparsers(dest="command")

        # Add subparsers for each command
        repl_parser = subparsers.add_parser("repl", help="Start Vira REPL")
        help_parser = subparsers.add_parser("help", help="Display list of commands")
        compile_parser = subparsers.add_parser("compile", help="Compile Vira code")
        compile_parser.add_argument("--platform", default=get_platform(), help="Target platform")
        compile_parser.add_argument("--output", default="build", help="Output directory")
        run_parser = subparsers.add_parser("run", help="Run Vira code in VM")
        run_parser.add_argument("file", help="File or directory to run")
        docs_parser = subparsers.add_parser("docs", help="Show documentation")
        init_parser = subparsers.add_parser("init", help="Initialize a new Vira project")
        install_parser = subparsers.add_parser("install", help="Install packages")
        install_parser.add_argument("packages", nargs="*", help="Packages to install")
        install_parser.add_argument("--in-project", action="store_true", help="Install in project")
        remove_parser = subparsers.add_parser("remove", help="Remove packages")
        remove_parser.add_argument("packages", nargs="+", help="Packages to remove")
        update_parser = subparsers.add_parser("update", help="Update packages")
        upgrade_parser = subparsers.add_parser("upgrade", help="Upgrade Vira language binaries")
        updatq_parser = subparsers.add_parser("updat?", help="Update Vira binaries and libraries")
        refresh_parser = subparsers.add_parser("refresh", help="Refresh repository cache")
        test_parser = subparsers.add_parser("test", help="Run tests")
        check_parser = subparsers.add_parser("check", help="Check .vira code and bytes.yml")
        fmt_parser = subparsers.add_parser("fmt", help="Format code")
        clean_parser = subparsers.add_parser("clean", help="Clean build artifacts")
        clean_parser.add_argument("--global", action="store_true", help="Clean global cache")
        search_parser = subparsers.add_parser("search", help="Search for libraries in repo")
        search_parser.add_argument("query", nargs="+", help="Search query")
        tutorial_parser = subparsers.add_parser("tutorial", help="Interactive tutorial")
        version_parser = subparsers.add_parser("version", help="Show Vira version")
        version_bytes_parser = subparsers.add_parser("version-bytes", help="Show bytes.io repository version")
        doctor_parser = subparsers.add_parser("doctor", help="Check environment and configuration")

        self.commands = {
            "repl": self.repl,
            "help": self.help_cmd,
            "compile": self.compile,
            "run": self.run,
            "docs": self.docs,
            "init": self.init,
            "install": self.install,
            "remove": self.remove,
            "update": self.update,
            "upgrade": self.upgrade,
            "updat?": self.updat_question,
            "refresh": self.refresh,
            "test": self.test,
            "check": self.check,
            "fmt": self.fmt,
            "clean": self.clean,
            "search": self.search,
            "tutorial": self.tutorial,
            "version": self.version,
            "version-bytes": self.version_bytes,
            "doctor": self.doctor,
        }

    def run(self):
        args = self.parser.parse_args()
        if args.help or not args.command:
            self.help_cmd()
            sys.exit(0)
        if args.verbose:
            logger.setLevel(logging.DEBUG)
            self.config["verbose"] = True
            save_vira_config(self.config)
        
        command_func = self.commands.get(args.command)
        if command_func:
            command_func(args)
        else:
            console.print(f"[bold red]Unknown command: {args.command}[/bold red]")
            self.help_cmd()
            sys.exit(1)

    def help_cmd(self, args=None):
        table = Table(title="Vira CLI Commands", show_header=True, header_style="bold magenta")
        table.add_column("Command", style="cyan")
        table.add_column("Description", style="green")
        
        for cmd in sorted(self.commands.keys()):
            table.add_row(cmd, self.commands[cmd].__doc__.strip() if self.commands[cmd].__doc__ else "No description")
        
        console.print(table)

    def repl(self, args):
        console.print("[bold green]Starting Vira REPL...[/bold green]")
        run_subprocess([str(VIRA_BIN / "vira-compiler"), "repl"])

    def compile(self, args):
        project_dir = Path.cwd()
        config = load_bytes_yml()
        resolve_dependencies(config)
        source_dir = project_dir / get_source_dir(config)
        output_dir = project_dir / args.output
        output_dir.mkdir(parents=True, exist_ok=True)
        
        with Progress(SpinnerColumn(), BarColumn(), TextColumn("[progress.description]{task.description}"), console=console) as progress:
            task = progress.add_task("Compiling...", total=100)
            cmd = [str(VIRA_BIN / "vira-compiler"), "compile", str(source_dir), "--platform", args.platform, "--output", str(output_dir)]
            run_subprocess(cmd)
            progress.advance(task, 100)
        
        console.print(f"[bold green]Compilation complete. Output in {args.output}/[/bold green]")

    def run(self, args):
        if not args.file:
            console.print("[bold red]Please provide a .vira file or directory to run.[/bold red]")
            sys.exit(1)
        
        run_subprocess([str(VIRA_BIN / "vira-compiler"), "run", args.file], timeout=300)

    def docs(self, args):
        console.print(Panel("Vira Documentation\n\n- Syntax: Use [ ] for blocks\n- Types: Static by default\nFor full docs, visit bytes.io", title="Docs"))

    def init(self, args):
        if find_bytes_yml():
            if not Confirm.ask("Project already initialized. Reinitialize?"):
                return
        
        project_name = Prompt.ask("Project name", default=Path.cwd().name)
        author = Prompt.ask("Author", default=os.getenv("USER", "unknown"))
        description = Prompt.ask("Description", default="")
        bytes_yml = {
            "name": project_name,
            "version": "0.1.0",
            "author": author,
            "description": description,
            "<>": "cmd",
            "dependencies": {},
            "dev-dependencies": {}
        }
        
        with open("bytes.yml", "w") as f:
            yaml.dump(bytes_yml, f)
            logger.info("Created bytes.yml")
        
        src_dir = Path.cwd() / "cmd"
        src_dir.mkdir(exist_ok=True)
        main_file = src_dir / "main.vira"
        with open(main_file, "w") as f:
            f.write("""<io>

@ Hello Vira program
func main()
[
    let msg: string = "Hello, Vira!"
    write msg
]
""")
            logger.info(f"Created {main_file}")
        
        test_dir = Path.cwd() / "tests"
        test_dir.mkdir(exist_ok=True)
        
        console.print("[bold green]Project initialized successfully.[/bold green]")

    def install(self, args):
        if not args.packages:
            config = load_bytes_yml()
            deps = config.get("dependencies", {})
            if deps:
                args.packages = [f"{dep}@{ver}" for dep, ver in deps.items()]
            else:
                console.print("[bold red]No packages specified and no dependencies in bytes.yml.[/bold red]")
                sys.exit(1)
        
        for pkg in args.packages:
            with Progress(SpinnerColumn(), TextColumn(f"Installing {pkg}..."), console=console) as progress:
                task = progress.add_task("", total=None)
                cmd = [str(VIRA_BIN / "vira-packages"), "install", pkg]
                if args.in_project:
                    cmd.append("--in-project")
                run_subprocess(cmd)
                progress.update(task, completed=True)
        
        console.print("[bold green]Installation complete.[/bold green]")

    def remove(self, args):
        for pkg in args.packages:
            run_subprocess([str(VIRA_BIN / "vira-packages"), "remove", pkg])
        
        console.print("[bold green]Removal complete.[/bold green]")

    def update(self, args):
        run_subprocess([str(VIRA_BIN / "vira-packages"), "update"])
        console.print("[bold green]Packages updated.[/bold green]")

    def upgrade(self, args):
        run_subprocess([str(VIRA_BIN / "vira-packages"), "upgrade"])
        console.print("[bold green]Vira upgraded.[/bold green]")

    def updat_question(self, args):
        self.update(None)
        self.upgrade(None)
        console.print("[bold green]Full update complete.[/bold green]")

    def refresh(self, args):
        run_subprocess([str(VIRA_BIN / "vira-packages"), "refresh"])
        console.print("[bold green]Repository refreshed.[/bold green]")

    def test(self, args):
        config = load_bytes_yml()
        test_dir = Path.cwd() / config.get("test_dir", "tests")
        if not test_dir.exists():
            console.print("[bold red]Tests directory not found.[/bold red]")
            sys.exit(1)
        with Progress(SpinnerColumn(), TextColumn("Running tests..."), console=console) as progress:
            task = progress.add_task("", total=None)
            run_subprocess([str(VIRA_BIN / "vira-compiler"), "test", str(test_dir)])
            progress.update(task, completed=True)
        console.print("[bold green]Tests complete.[/bold green]")

    def check(self, args):
        config = load_bytes_yml()
        required_keys = ["name", "version"]
        missing = [k for k in required_keys if k not in config]
        if missing:
            console.print(f"[bold red]Invalid bytes.yml: missing {', '.join(missing)}.[/bold red]")
            sys.exit(1)
        
        project_dir = Path.cwd()
        source_dir = project_dir / get_source_dir(config)
        if not source_dir.exists():
            console.print("[bold red]Source directory not found.[/bold red]")
            sys.exit(1)
        
        files = list(source_dir.rglob("*.vira"))
        with Progress(SpinnerColumn(), BarColumn(), TextColumn("Checking files..."), console=console) as progress:
            task = progress.add_task("", total=len(files))
            for file in files:
                run_subprocess([str(VIRA_BIN / "vira-parser_lexer"), "check", str(file)])
                progress.advance(task)
        
        console.print("[bold green]All checks passed.[/bold green]")

    def fmt(self, args):
        project_dir = Path.cwd()
        config = load_bytes_yml()
        source_dir = project_dir / get_source_dir(config)
        files = list(source_dir.rglob("*.vira"))
        with Progress(SpinnerColumn(), BarColumn(), TextColumn("Formatting files..."), console=console) as progress:
            task = progress.add_task("", total=len(files))
            for file in files:
                run_subprocess([str(VIRA_BIN / "vira-parser_lexer"), "fmt", str(file)])
                progress.advance(task)
        
        console.print("[bold green]Formatting complete.[/bold green]")

    def clean(self, args):
        build_dir = Path.cwd() / "build"
        if build_dir.exists() and Confirm.ask(f"Clean project build directory {build_dir}?", default=True):
            shutil.rmtree(build_dir)
            logger.info(f"Cleaned {build_dir}")
        
        if args.global and Confirm.ask("Clean global cache and logs?", default=False):
            if VIRA_CACHE.exists():
                shutil.rmtree(VIRA_CACHE)
                VIRA_CACHE.mkdir()
                logger.info("Cleaned global cache")
            if VIRA_LOGS.exists():
                for log in VIRA_LOGS.glob("*"):
                    log.unlink()
                logger.info("Cleaned logs")
        
        console.print("[bold green]Clean complete.[/bold green]")

    def search(self, args):
        query = " ".join(args.query)
        output = run_subprocess([str(VIRA_BIN / "vira-packages"), "search", query], capture_output=True)
        console.print(Panel(output.strip(), title="Search Results", expand=False))

    def tutorial(self, args):
        console.print("[bold green]Welcome to Vira Interactive Tutorial![/bold green]")
        lessons = [
            ("Lesson 1: Hello World", """func main() [ write "Hello, Vira!" ]""", "Write a simple hello world."),
            ("Lesson 2: Variables and Types", """let x: int = 42\nlet y: string = "Answer"\nwrite y + " is " + x""", "Declare variables with types."),
            ("Lesson 3: Functions and Recursion", """func factorial(n: int) -> int [\n    if n <= 1 [ return 1 ]\n    return n * factorial(n - 1)\n]\nwrite factorial(5)""", "Define a recursive function."),
        ]
        for title, code, hint in lessons:
            console.print(Panel(code, title=title))
            console.print(f"[italic]{hint}[/italic]")
            while True:
                user_code = Prompt.ask("Your code (or 'skip')")
                if user_code.lower() == "skip":
                    break
                # Simulate execution via compiler eval if possible
                try:
                    output = run_subprocess([str(VIRA_BIN / "vira-compiler"), "eval", user_code], capture_output=True)
                    console.print(f"[green]Output: {output.strip()}[/green]")
                    break
                except:
                    console.print("[bold red]Error in code. Try again.[/bold red]")
        console.print("[bold green]Tutorial complete! You're ready to code in Vira.[/bold green]")

    def version(self, args):
        version = self.config.get("version", "0.1.0")
        console.print(f"Vira CLI version: {version}")

    def version_bytes(self, args):
        # Simulate fetch from repo
        output = run_subprocess([str(VIRA_BIN / "vira-packages"), "version-bytes"], capture_output=True)
        console.print(f"bytes.io version: {output.strip() or 'Unknown'}")

    def doctor(self, args):
        table = Table(title="Vira Doctor Report", show_header=True, header_style="bold magenta")
        table.add_column("Check", style="cyan")
        table.add_column("Status", style="green")
        table.add_column("Details", style="yellow")
        
        checks = [
            ("VIRA_HOME", VIRA_HOME.exists(), str(VIRA_HOME)),
            ("VIRA_BIN", VIRA_BIN.exists(), str(VIRA_BIN)),
            ("vira-compiler", (VIRA_BIN / "vira-compiler").exists(), "Compiler binary"),
            ("vira-packages", (VIRA_BIN / "vira-packages").exists(), "Packages binary"),
            ("vira-parser_lexer", (VIRA_BIN / "vira-parser_lexer").exists(), "Parser/Lexer binary"),
            ("Python version", sys.version_info >= (3, 8), sys.version.split()[0]),
            ("YAML config", VIRA_CONFIG.exists(), "Global config"),
        ]
        
        all_passed = True
        for check, status, details in checks:
            status_text = "[green]OK[/green]" if status else "[red]FAIL[/red]"
            table.add_row(check, status_text, details)
            if not status:
                all_passed = False
        
        console.print(table)
        if all_passed:
            console.print("[bold green]System is healthy.[/bold green]")
        else:
            console.print("[bold red]Issues detected. Please resolve FAIL items.[/bold red]")

if __name__ == "__main__":
    cli = ViraCLI()
    cli.run()
