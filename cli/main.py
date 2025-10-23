import argparse
import sys
from .config import load_vira_config, save_vira_config, logger
from .commands import help_cmd, repl, compile_cmd, run_cmd, docs_cmd, init_cmd, install_cmd # etc.

class ViraCLI:
    def __init__(self):
        self.config = load_vira_config()
        if self.config.get("verbose"):
            logger.setLevel(logging.DEBUG)
        self.parser = argparse.ArgumentParser(description="Vira CLI", add_help=False)
        self.parser.add_argument("-v", "--verbose", action="store_true", help="Enable verbose mode")
        self.parser.add_argument("-h", "--help", action="store_true", help="Show help")
        subparsers = self.parser.add_subparsers(dest="command")

        # Add subparsers
        subparsers.add_parser("repl", help="Start Vira REPL")
        subparsers.add_parser("help", help="Display list of commands")
        compile_parser = subparsers.add_parser("compile", help="Compile Vira code")
        compile_parser.add_argument("--platform", default=get_platform(), help="Target platform")
        compile_parser.add_argument("--output", default="build", help="Output directory")
        run_parser = subparsers.add_parser("run", help="Run Vira code in VM")
        run_parser.add_argument("file", help="File or directory to run")
        subparsers.add_parser("docs", help="Show documentation")
        subparsers.add_parser("init", help="Initialize a new Vira project")
        install_parser = subparsers.add_parser("install", help="Install packages")
        install_parser.add_argument("packages", nargs="*", help="Packages to install")
        install_parser.add_argument("--in-project", action="store_true", help="Install in project")
        # Add all other parsers...

    def run(self):
        args = self.parser.parse_args()
        if args.help or not args.command:
            help_cmd()
            sys.exit(0)
        if args.verbose:
            logger.setLevel(logging.DEBUG)
            self.config["verbose"] = True
            save_vira_config(self.config)
        
        command_map = {
            "repl": repl,
            "help": help_cmd,
            "compile": compile_cmd,
            "run": run_cmd,
            "docs": docs_cmd,
            "init": init_cmd,
            "install": install_cmd,
            # Map all commands
        }
        
        command_func = command_map.get(args.command)
        if command_func:
            command_func(args)
        else:
            console.print(f"[bold red]Unknown command: {args.command}[/bold red]")
            help_cmd()
            sys.exit(1)

if __name__ == "__main__":
    cli = ViraCLI()
    cli.run()
