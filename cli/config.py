import os
import yaml
from pathlib import Path
import logging

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

def save_vira_config(config: dict):
    with open(VIRA_CONFIG, "w") as f:
        yaml.dump(config, f)
    logger.info("Saved Vira config")

def load_vira_config() -> dict:
    if VIRA_CONFIG.exists():
        with open(VIRA_CONFIG, "r") as f:
            return yaml.safe_load(f) or {}
    default_config = {"version": "0.1.0", "verbose": False}
    save_vira_config(default_config)
    return default_config

def load_bytes_yml(path: Path | None = None) -> dict:
    if path is None:
        path = find_bytes_yml()
    if path:
        try:
            with open(path, "r") as f:
                config = yaml.safe_load(f)
                logger.info(f"Loaded config from {path}")
                return config or {}
        except yaml.YAMLError as e:
            logger.error(f"YAML error in {path}: {e}")
            raise
    return {}

def find_bytes_yml(start_dir: Path = Path.cwd()) -> Path | None:
    current = start_dir
    while current != current.parent:
        bytes_path = current / "bytes.yml"
        if bytes_path.exists():
            return bytes_path
        current = current.parent
    return None

def get_source_dir(config: dict) -> str:
    return config.get("<>", "cmd") or "cmd"

def get_platform() -> str:
    platforms = {
        "linux": "linux",
        "win32": "windows",
        "darwin": "macos"
    }
    return platforms.get(os.sys.platform, "unknown")
