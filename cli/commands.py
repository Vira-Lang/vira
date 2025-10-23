import os
import shutil
from pathlib import Path
from typing import Any
from rich.panel import Panel
from rich.prompt import Prompt, Confirm
from rich.table import Table
from rich.progress import Progress, SpinnerColumn, TextColumn, BarColumn
from .config import load_bytes_yml, get_source_dir, logger, VIRA_BIN, VIRA_CACHE, VIRA_LOGS
from .utils import run_subprocess, resolve_dependencies, console

def help_cmd():
    table = Table(title="Vira CLI Commands", show_header=True, header_style="bold magenta")
    table.add_column("Command", style="cyan")
    table.add_column("Description", style="green")
    
    # List all commands dynamically if possible, or hardcode
    commands_desc = {
        "repl": "Start Vira REPL",
        "compile": "Compile Vira code",
        # Add all others...
    }
    for cmd, desc in sorted(commands_desc.items()):
        table.add_row(cmd, desc)
    
    console.print(table)

def repl(args: Any):
    console.print("[bold green]Starting Vira REPL...[/bold green]")
    run_subprocess([str(VIRA_BIN / "vira-compiler"), "repl"])

def compile_cmd(args: Any):
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

def run_cmd(args: Any):
    if not args.file:
        console.print("[bold red]Please provide a .vira file or directory to run.[/bold red]")
        sys.exit(1)
    
    run_subprocess([str(VIRA_BIN / "vira-compiler"), "run", args.file], timeout=300)

def docs_cmd(args: Any):
    console.print(Panel("Vira Documentation\n\n- Syntax: Use [ ] for blocks\n- Types: Static by default\nFor full docs, visit bytes.io", title="Docs"))

def init_cmd(args: Any):
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

# Add other command functions similarly: install_cmd, remove_cmd, etc.

def install_cmd(args: Any):
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

# ... Continue for all commands
