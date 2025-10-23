import subprocess
import sys
from typing import List, Optional
from rich.console import Console

console = Console()

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

def resolve_dependencies(config: dict):
    deps = config.get("dependencies", {})
    for dep, version in deps.items():
        dep_path = VIRA_LIBS / f"{dep}-{version}"
        if not dep_path.exists():
            console.print(f"[bold yellow]Installing missing dependency: {dep}@{version}[/bold yellow]")
            run_subprocess([str(VIRA_BIN / "vira-packages"), "install", f"{dep}@{version}"])
