const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const yaml = require('js-yaml');
const ora = require('ora');
const chalk = require('chalk');
const rimraf = require('rimraf');

let constants = {};

function setConstants(consts) {
    constants = consts;
}

function runSubprocess(cmd, captureOutput = false, timeout = null) {
    try {
        const options = { timeout, stdio: captureOutput ? 'pipe' : 'inherit' };
        if (captureOutput) {
            return execSync(cmd.join(' '), options).toString().trim();
        } else {
            execSync(cmd.join(' '), options);
        }
    } catch (e) {
        console.log(chalk.red.bold(`Error executing command: ${e.message}`));
        process.exit(1);
    }
    return '';
}

function getPlatform() {
    const platforms = { linux: 'linux', win32: 'windows', darwin: 'macos' };
    return platforms[process.platform] || 'unknown';
}

function findBytesYml(startDir = process.cwd()) {
    let current = startDir;
    while (current !== path.parse(current).root) {
        const bytesPath = path.join(current, 'bytes.yml');
        if (fs.existsSync(bytesPath)) return bytesPath;
        current = path.dirname(current);
    }
    return null;
}

function loadBytesYml(p = null) {
    p = p || findBytesYml();
    if (p) {
        return yaml.load(fs.readFileSync(p, 'utf8')) || {};
    }
    console.log(chalk.yellow.bold('No bytes.yml found. Using defaults.'));
    return {};
}

function saveViraConfig(config) {
    fs.writeFileSync(constants.VIRA_CONFIG, yaml.dump(config));
}

function loadViraConfig() {
    if (fs.existsSync(constants.VIRA_CONFIG)) {
        return yaml.load(fs.readFileSync(constants.VIRA_CONFIG, 'utf8')) || {};
    }
    const defaultConfig = { version: '0.1.0', verbose: false };
    saveViraConfig(defaultConfig);
    return defaultConfig;
}

function resolveDependencies(config) {
    const deps = config.dependencies || {};
    Object.entries(deps).forEach(([dep, version]) => {
        const depPath = path.join(constants.VIRA_LIBS, `${dep}-${version}`);
        if (!fs.existsSync(depPath)) {
            console.log(chalk.yellow.bold(`Installing missing dependency: ${dep}@${version}`));
            runSubprocess([path.join(constants.VIRA_BIN, 'vira-packages'), 'install', `${dep}@${version}`]);
        }
    });
}

function getFiles(dir, ext) {
    let files = [];
    fs.readdirSync(dir, { withFileTypes: true }).forEach(item => {
        const itemPath = path.join(dir, item.name);
        if (item.isDirectory()) {
            files = files.concat(getFiles(itemPath, ext));
        } else if (path.extname(item.name) === ext) {
            files.push(itemPath);
        }
    });
    return files;
}

function startSpinner(text) {
    return ora(chalk.cyan(text)).start();
}

module.exports = {
    setConstants,
    runSubprocess,
    getPlatform,
    findBytesYml,
    loadBytesYml,
    saveViraConfig,
    loadViraConfig,
    resolveDependencies,
    getFiles,
    startSpinner,
    constants: () => constants // Getter for constants
};
