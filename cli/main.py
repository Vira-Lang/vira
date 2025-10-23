# Expanded Python CLI for Vira
# Now with more robust error handling, logging, configuration loading, and additional features

import os
import sys
import subprocess
import shutil
import yaml
import logging
from pathlib import Path
from typing import List, Dict
from rich.console import Console
from rich.panel import Panel
from rich.prompt import Prompt, Confirm
from rich.table import Table
from rich.progress import Progress, SpinnerColumn, TextColumn
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

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    handlers=[
        logging.FileHandler(VIRA_LOGS / "vira.log"),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger("vira")

def run_subprocess(cmd: List[str], capture_output: bool = False) -> str:
    try:
        if capture_output:
            output = subprocess.check_output(cmd, stderr=subprocess.STDOUT).decode()
            logger.info(f"Command output: {output}")
            return output
        else:
            subprocess.check_call(cmd)
            logger.info(f"Command {cmd} executed successfully")
    except subprocess.CalledProcessError as e:
        error_msg = e.output.decode() if e.output else str(e)
        console.print(f"[bold red]Error:[/bold red] {error_msg}")
        logger.error(f"Error running {cmd}: {error_msg}")
        sys.exit(1)
    return ""

def get_platform() -> str:
    platforms = {
        "linux": "linux",
        "win32": "windows",
        "darwin": "macos"
    }
    return platforms.get(sys.platform, "unknown")

def find_bytes_yml() -> Path | None:
    current_dir = Path.cwd()
    bytes_path = current_dir / "bytes.yml"
    if bytes_path.exists():
        return bytes_path
    for path in current_dir.rglob("bytes.yml"):
        if path.exists():
            return path
    return None

def load_bytes_yml() -> Dict:
    bytes_path = find_bytes_yml()
    if bytes_path:
        try:
            with open(bytes_path, "r") as f:
                config = yaml.safe_load(f)
                logger.info(f"Loaded config from {bytes_path}")
                return config or {}
        except yaml.YAMLError as e:
            console.print(f"[bold red]Invalid YAML in bytes.yml:[/bold red] {e}")
            logger.error(f"YAML error: {e}")
            sys.exit(1)
    return {}

def get_source_dir(config: Dict) -> str:
    return config.get("<>", "cmd") or "cmd"

def load_vira_config() -> Dict:
    if VIRA_CONFIG.exists():
        with open(VIRA_CONFIG, "r") as f:
            return yaml.safe_load(f) or {}
    return {}

class ViraCLI:
    def __init__(self):
        self.config = load_vira_config()
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
        if len(sys.argv) < 2:
            self.help_cmd()
            sys.exit(0)
        
        cmd = sys.argv[1]
        args = sys.argv[2:]
        
        if cmd in self.commands:
            self.commands[cmd](args)
        else:
            console.print(f"[bold red]Unknown command:[/bold red] {cmd}")
            self.help_cmd()
            sys.exit(1)

    def help_cmd(self, args: List[str] = None):
        """Display list of commands"""
        table = Table(title="Vira CLI Commands", show_header=True, header_style="bold magenta")
        table.add_column("Command", style="cyan")
        table.add_column("Description", style="green")
        
        for cmd, func in sorted(self.commands.items()):
            doc = func.__doc__.strip() if func.__doc__ else "No description available"
            table.add_row(cmd, doc)
        
        console.print(table)

    def repl(self, args: List[str]):
        """Start Vira REPL"""
        console.print("[bold green]Starting Vira REPL...[/bold green]")
        run_subprocess([str(VIRA_BIN / "vira-compiler"), "repl"])

    def compile(self, args: List[str]):
        """Compile Vira code"""
        platform = get_platform()
        for i, arg in enumerate(args):
            if arg.startswith("--"):
                platform = arg[2:]
                args.pop(i)
                break
        
        project_dir = Path.cwd()
        config = load_bytes_yml()
        source_dir = project_dir / get_source_dir(config)
        output_dir = project_dir / "build"
        output_dir.mkdir(parents=True, exist_ok=True)
        
        with Progress(SpinnerColumn(), TextColumn("[progress.description]{task.description}"), console=console, transient=True) as progress:
            task = progress.add_task("Compiling...", total=None)
            cmd = [str(VIRA_BIN / "vira-compiler"), "compile", str(source_dir), "--platform", platform, "--output", str(output_dir)]
            run_subprocess(cmd)
            progress.update(task, completed=True)
        
        console.print("[bold green]Compilation complete. Output in build/[/bold green]")

    def run(self, args: List[str]):
        """Run Vira code in VM"""
        if not args:
            console.print("[bold red]Please provide a .vira file or directory to run.[/bold red]")
            sys.exit(1)
        
        file_or_dir = args[0]
        run_subprocess([str(VIRA_BIN / "vira-compiler"), "run", file_or_dir])

    def docs(self, args: List[str]):
        """Show documentation"""
        console.print(Panel("Vira Documentation\n\nVira is a futuristic, memory-safe language.\nFor more, check online resources or bytes.io", title="Docs"))

    def init(self, args: List[str]):
        """Initialize a new Vira project"""
        project_name = Prompt.ask("Project name", default=Path.cwd().name)
        author = Prompt.ask("Author", default=os.getenv("USER", "unknown"))
        bytes_yml = {
            "name": project_name,
            "version": "0.1.0",
            "author": author,
            "<>": "cmd",
            "dependencies": {}
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
    write "Hello, Vira!"
]
""")
            logger.info(f"Created {main_file}")
        
        console.print("[bold green]Project initialized successfully.[/bold green]")

    def install(self, args: List[str]):
        """Install packages"""
        in_project = "--in-project" in args
        args = [a for a in args if a != "--in-project"]
        
        if not args:
            console.print("[bold red]Please provide package names to install.[/bold red]")
            sys.exit(1)
        
        for pkg in args:
            with Progress(SpinnerColumn(), TextColumn(f"Installing {pkg}..."), console=console, transient=True) as progress:
                task = progress.add_task("", total=None)
                cmd = [str(VIRA_BIN / "vira-packages"), "install", pkg]
                if in_project:
                    cmd.append("--in-project")
                run_subprocess(cmd)
                progress.update(task, completed=True)
        
        console.print("[bold green]Installation complete.[/bold green]")

    def remove(self, args: List[str]):
        """Remove packages"""
        if not args:
            console.print("[bold red]Please provide package names to remove.[/bold red]")
            sys.exit(1)
        
        for pkg in args:
            run_subprocess([str(VIRA_BIN / "vira-packages"), "remove", pkg])
        
        console.print("[bold green]Removal complete.[/bold green]")

    def update(self, args: List[str]):
        """Update packages"""
        run_subprocess([str(VIRA_BIN / "vira-packages"), "update"])
        console.print("[bold green]Packages updated.[/bold green]")

    def upgrade(self, args: List[str]):
        """Upgrade Vira language binaries"""
        run_subprocess([str(VIRA_BIN / "vira-packages"), "upgrade"])
        console.print("[bold green]Vira upgraded.[/bold green]")

    def updat_question(self, args: List[str]):
        """Update Vira binaries and libraries"""
        self.update([])
        self.upgrade([])
        console.print("[bold green]Full update complete.[/bold green]")

    def refresh(self, args: List[str]):
        """Refresh repository cache"""
        run_subprocess([str(VIRA_BIN / "vira-packages"), "refresh"])
        console.print("[bold green]Repository refreshed.[/bold green]")

    def test(self, args: List[str]):
        """Run tests"""
        config = load_bytes_yml()
        test_dir = Path.cwd() / config.get("test_dir", "tests")
        if not test_dir.exists():
            console.print("[bold red]Tests directory not found.[/bold red]")
            sys.exit(1)
        run_subprocess([str(VIRA_BIN / "vira-compiler"), "test", str(test_dir)])
        console.print("[bold green]Tests complete.[/bold green]")

    def check(self, args: List[str]):
        """Check .vira code and bytes.yml"""
        config = load_bytes_yml()
        if "name" not in config or "version" not in config:
            console.print("[bold red]Invalid bytes.yml: missing name or version.[/bold red]")
            sys.exit(1)
        
        project_dir = Path.cwd()
        source_dir = project_dir / get_source_dir(config)
        if not source_dir.exists():
            console.print("[bold red]Source directory not found.[/bold red]")
            sys.exit(1)
        
        for file in source_dir.rglob("*.vira"):
            run_subprocess([str(VIRA_BIN / "vira-parser_lexer"), "check", str(file)])
        
        console.print("[bold green]Check passed.[/bold green]")

    def fmt(self, args: List[str]):
        """Format code"""
        project_dir = Path.cwd()
        config = load_bytes_yml()
        source_dir = project_dir / get_source_dir(config)
        for file in source_dir.rglob("*.vira"):
            run_subprocess([str(VIRA_BIN / "vira-parser_lexer"), "fmt", str(file)])
        
        console.print("[bold green]Formatting complete.[/bold green]")

    def clean(self, args: List[str]):
        """Clean build artifacts"""
        build_dir = Path.cwd() / "build"
        cache_dir = VIRA_CACHE
        if Confirm.ask(f"Clean build directory {build_dir}?", default=True):
            if build_dir.exists():
                shutil.rmtree(build_dir)
                logger.info(f"Cleaned {build_dir}")
        if Confirm.ask("Clean global cache?", default=False):
            if cache_dir.exists():
                shutil.rmtree(cache_dir)
                cache_dir.mkdir()
                logger.info("Cleaned global cache")
        console.print("[bold green]Clean complete.[/bold green]")

    def search(self, args: List[str]):
        """Search for libraries in repo"""
        if not args:
            console.print("[bold red]Please provide search query.[/bold red]")
            sys.exit(1)
        query = " ".join(args)
        output = run_subprocess([str(VIRA_BIN / "vira-packages"), "search", query], capture_output=True)
        console.print(Panel(output.strip(), title="Search Results", expand=False))

    def tutorial(self, args: List[str]):
        """Interactive tutorial"""
        console.print("[bold green]Welcome to Vira Tutorial![/bold green]")
        lessons = [
            ("Lesson 1: Hello World", """func main() [ write "Hello, Vira!" ]"""),
            ("Lesson 2: Variables", """let x: int = 5\nwrite x"""),
            ("Lesson 3: Functions", """func add(a: int, b: int) -> int [ return a + b ]""")
        ]
        for title, code in lessons:
            console.print(Panel(code, title=title))
            user_code = Prompt.ask("Try writing the code")
            # Simulate check
            if user_code.strip() == code.strip():
                console.print("[green]Correct![/green]")
            else:
                console.print("[yellow]Close, try again.[/yellow]")
        console.print("[bold green]Tutorial complete![/bold green]")

    def version(self, args: List[str]):
        """Show Vira version"""
        version = self.config.get("version", "0.1.0")
        console.print(f"Vira version: {version}")

    def version_bytes(self, args: List[str]):
        """Show bytes.io repository version"""
        # Simulate fetch
        console.print("bytes.io version: 1.2.3")

    def doctor(self, args: List[str]):
        """Check environment and configuration"""
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
            ("Python version", sys.version_info >= (3, 8), sys.version),
        ]
        
        for check, status, details in checks:
            status_text = "[green]OK[/green]" if status else "[red]FAIL[/red]"
            table.add_row(check, status_text, details)
        
        console.print(table)
        if all(status for _, status, _ in checks):
            console.print("[bold green]All checks passed.[/bold green]")
        else:
            console.print("[bold red]Some checks failed. Please fix.[/bold red]")

if __name__ == "__main__":
    cli = ViraCLI()
    cli.run()
