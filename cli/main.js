const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const commander = require('commander');
const inquirer = require('inquirer');
const chalk = require('chalk');
const Table = require('cli-table3');
const ora = require('ora');
const yaml = require('js-yaml');
const rimraf = require('rimraf');

const VIRA_HOME = path.join(process.env.HOME || process.env.USERPROFILE, '.vira');
const VIRA_BIN = path.join(VIRA_HOME, 'bin');
const VIRA_LIBS = path.join(VIRA_HOME, 'libs');
const VIRA_LOGS = path.join(VIRA_HOME, 'logs');
const VIRA_CACHE = path.join(VIRA_HOME, 'cache');
const VIRA_CONFIG = path.join(VIRA_HOME, 'config.yml');

// Ensure directories exist
[VIRA_HOME, VIRA_BIN, VIRA_LIBS, VIRA_LOGS, VIRA_CACHE].forEach(dir => {
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
});

function runSubprocess(cmd, captureOutput = false, timeout = null) {
  try {
    const options = { timeout, stdio: captureOutput ? 'pipe' : 'inherit' };
    if (captureOutput) {
      return execSync(cmd.join(' '), options).toString().trim();
    } else {
      execSync(cmd.join(' '), options);
    }
  } catch (e) {
    console.log(chalk.red(`Error: ${e.message}`));
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
  console.log(chalk.yellow('No bytes.yml found. Using defaults.'));
  return {};
}

function saveViraConfig(config) {
  fs.writeFileSync(VIRA_CONFIG, yaml.dump(config));
}

function loadViraConfig() {
  if (fs.existsSync(VIRA_CONFIG)) {
    return yaml.load(fs.readFileSync(VIRA_CONFIG, 'utf8')) || {};
  }
  const defaultConfig = { version: '0.1.0', verbose: false };
  saveViraConfig(defaultConfig);
  return defaultConfig;
}

function resolveDependencies(config) {
  const deps = config.dependencies || {};
  Object.entries(deps).forEach(([dep, version]) => {
    const depPath = path.join(VIRA_LIBS, `${dep}-${version}`);
    if (!fs.existsSync(depPath)) {
      console.log(chalk.yellow(`Installing missing dependency: ${dep}@${version}`));
      runSubprocess([path.join(VIRA_BIN, 'vira-packages'), 'install', `${dep}@${version}`]);
    }
  });
}

const program = new commander.Command('vira')
  .description('Vira CLI')
  .version('0.1.0')
  .option('-v, --verbose', 'Enable verbose mode');

program.command('repl')
  .description('Start Vira REPL')
  .action(() => {
    console.log(chalk.yellow('REPL placeholder: Starting Vira REPL...'));
    // runSubprocess([path.join(VIRA_BIN, 'vira-compiler'), 'repl']);
  });

program.command('help')
  .description('Display list of commands')
  .action(() => {
    const table = new Table({ head: ['Command', 'Description'], style: { head: ['magenta'] } });
    program.commands.forEach(cmd => {
      table.push([chalk.cyan(cmd.name()), chalk.green(cmd.description())]);
    });
    console.log(table.toString());
  });

program.command('compile')
  .description('Compile Vira code')
  .option('--platform <plat>', 'Target platform', getPlatform())
  .option('--output <out>', 'Output directory', 'build')
  .action((cmd) => {
    const spinner = ora('Compiling...').start();
    const config = loadBytesYml();
    resolveDependencies(config);
    const sourceDir = path.join(process.cwd(), config['<>'] || 'cmd');
    const outputDir = path.join(process.cwd(), cmd.output);
    fs.mkdirSync(outputDir, { recursive: true });
    runSubprocess([path.join(VIRA_BIN, 'vira-compiler'), 'compile', sourceDir, '--platform', cmd.platform, '--output', outputDir]);
    spinner.succeed(chalk.green(`Compilation complete. Output in ${cmd.output}/`));
  });

program.command('run')
  .description('Run Vira code in VM')
  .argument('<file>', 'File or directory to run')
  .action((file) => {
    const spinner = ora('Running...').start();
    runSubprocess([path.join(VIRA_BIN, 'vira-compiler'), 'run', file], false, 300000);
    spinner.succeed(chalk.green('Run complete.'));
  });

program.command('docs')
  .description('Show documentation')
  .action(() => {
    const docs = `Vira Documentation

- Syntax: Use [ ] for blocks
- Types: Static by default
For full docs, visit bytes.io`;
    console.log(chalk.green(docs));
  });

program.command('init')
  .description('Initialize a new Vira project')
  .action(async () => {
    const existing = findBytesYml();
    if (existing) {
      const { reinit } = await inquirer.prompt([{ type: 'confirm', name: 'reinit', message: 'Project already initialized. Reinitialize?', default: false }]);
      if (!reinit) return;
    }
    const answers = await inquirer.prompt([
      { name: 'name', message: 'Project name', default: path.basename(process.cwd()) },
      { name: 'author', message: 'Author', default: process.env.USER || 'unknown' },
      { name: 'description', message: 'Description', default: '' },
    ]);
    const bytesYml = {
      name: answers.name,
      version: '0.1.0',
      author: answers.author,
      description: answers.description,
      '<>': 'cmd',
      dependencies: {},
      'dev-dependencies': {}
    };
    fs.writeFileSync('bytes.yml', yaml.dump(bytesYml));
    fs.mkdirSync('cmd', { recursive: true });
    fs.writeFileSync(path.join('cmd', 'main.vira'), `<io>

@ Hello Vira program
func main()
[
    let msg: string = "Hello, Vira!"
    write msg
]
`);
    fs.mkdirSync('tests', { recursive: true });
    console.log(chalk.green('Project initialized successfully.'));
  });

program.command('install')
  .description('Install packages')
  .argument('[packages...]', 'Packages to install')
  .option('--in-project', 'Install in project')
  .action(async (packages, options) => {
    if (!packages.length) {
      const config = loadBytesYml();
      const deps = config.dependencies || {};
      packages = Object.entries(deps).map(([dep, ver]) => `${dep}@${ver}`);
      if (!packages.length) {
        console.log(chalk.red('No packages specified and no dependencies in bytes.yml.'));
        process.exit(1);
      }
    }
    for (const pkg of packages) {
      const spinner = ora(`Installing ${pkg}...`).start();
      const cmd = [path.join(VIRA_BIN, 'vira-packages'), 'install', pkg];
      if (options.inProject) cmd.push('--in-project');
      runSubprocess(cmd);
      spinner.succeed(chalk.green(`Installed ${pkg}`));
    }
    console.log(chalk.green('Installation complete.'));
  });

program.command('remove')
  .description('Remove packages')
  .argument('<packages...>', 'Packages to remove')
  .action((packages) => {
    for (const pkg of packages) {
      const spinner = ora(`Removing ${pkg}...`).start();
      runSubprocess([path.join(VIRA_BIN, 'vira-packages'), 'remove', pkg]);
      spinner.succeed(chalk.green(`Removed ${pkg}`));
    }
    console.log(chalk.green('Removal complete.'));
  });

program.command('update')
  .description('Update packages')
  .action(() => {
    const spinner = ora('Updating packages...').start();
    runSubprocess([path.join(VIRA_BIN, 'vira-packages'), 'update']);
    spinner.succeed(chalk.green('Packages updated.'));
  });

program.command('upgrade')
  .description('Upgrade Vira language binaries')
  .action(() => {
    const spinner = ora('Upgrading Vira...').start();
    runSubprocess([path.join(VIRA_BIN, 'vira-packages'), 'upgrade']);
    spinner.succeed(chalk.green('Vira upgraded.'));
  });

program.command('updat?')
  .description('Update Vira binaries and libraries')
  .action(() => {
    program.commands.find(c => c.name() === 'update').action()();
    program.commands.find(c => c.name() === 'upgrade').action()();
    console.log(chalk.green('Full update complete.'));
  });

program.command('refresh')
  .description('Refresh repository cache')
  .action(() => {
    const spinner = ora('Refreshing repository...').start();
    runSubprocess([path.join(VIRA_BIN, 'vira-packages'), 'refresh']);
    spinner.succeed(chalk.green('Repository refreshed.'));
  });

program.command('test')
  .description('Run tests')
  .action(() => {
    const config = loadBytesYml();
    const testDir = path.join(process.cwd(), config.test_dir || 'tests');
    if (!fs.existsSync(testDir)) {
      console.log(chalk.red('Tests directory not found.'));
      process.exit(1);
    }
    const spinner = ora('Running tests...').start();
    runSubprocess([path.join(VIRA_BIN, 'vira-compiler'), 'test', testDir]);
    spinner.succeed(chalk.green('Tests complete.'));
  });

program.command('check')
  .description('Check .vira code and bytes.yml')
  .action(() => {
    const config = loadBytesYml();
    const requiredKeys = ['name', 'version'];
    const missing = requiredKeys.filter(k => !(k in config));
    if (missing.length) {
      console.log(chalk.red(`Invalid bytes.yml: missing ${missing.join(', ')}.`));
      process.exit(1);
    }
    const sourceDir = path.join(process.cwd(), config['<>'] || 'cmd');
    if (!fs.existsSync(sourceDir)) {
      console.log(chalk.red('Source directory not found.'));
      process.exit(1);
    }
    const files = getFiles(sourceDir, '.vira');
    const spinner = ora('Checking files...').start();
    files.forEach(file => {
      runSubprocess([path.join(VIRA_BIN, 'vira-parser_lexer'), 'check', file]);
    });
    spinner.succeed(chalk.green('All checks passed.'));
  });

program.command('fmt')
  .description('Format code')
  .action(() => {
    const config = loadBytesYml();
    const sourceDir = path.join(process.cwd(), config['<>'] || 'cmd');
    const files = getFiles(sourceDir, '.vira');
    const spinner = ora('Formatting files...').start();
    files.forEach(file => {
      runSubprocess([path.join(VIRA_BIN, 'vira-parser_lexer'), 'fmt', file]);
    });
    spinner.succeed(chalk.green('Formatting complete.'));
  });

program.command('clean')
  .description('Clean build artifacts')
  .option('--global', 'Clean global cache')
  .action(async (options) => {
    const buildDir = path.join(process.cwd(), 'build');
    if (fs.existsSync(buildDir)) {
      const { confirm } = await inquirer.prompt([{ type: 'confirm', name: 'confirm', message: `Clean ${buildDir}?`, default: true }]);
      if (confirm) {
        rimraf.sync(buildDir);
        console.log(chalk.green(`Cleaned ${buildDir}`));
      }
    }
    if (options.global) {
      const { confirmGlobal } = await inquirer.prompt([{ type: 'confirm', name: 'confirmGlobal', message: 'Clean global cache and logs?', default: false }]);
      if (confirmGlobal) {
        rimraf.sync(VIRA_CACHE);
        fs.mkdirSync(VIRA_CACHE);
        console.log(chalk.green('Cleaned global cache'));
        fs.readdirSync(VIRA_LOGS).forEach(file => fs.unlinkSync(path.join(VIRA_LOGS, file)));
        console.log(chalk.green('Cleaned logs'));
      }
    }
    console.log(chalk.green('Clean complete.'));
  });

program.command('search')
  .description('Search for libraries in repo')
  .argument('<query...>', 'Search query')
  .action((query) => {
    const output = runSubprocess([path.join(VIRA_BIN, 'vira-packages'), 'search', query.join(' ')], true);
    console.log(chalk.green('Search Results:\n') + output);
  });

program.command('tutorial')
  .description('Interactive tutorial')
  .action(async () => {
    console.log(chalk.green('Welcome to Vira Interactive Tutorial!'));
    const lessons = [
      { title: 'Lesson 1: Hello World', code: 'func main() [ write "Hello, Vira!" ]', hint: 'Write a simple hello world.' },
      { title: 'Lesson 2: Variables and Types', code: 'let x: int = 42\nlet y: string = "Answer"\nwrite y + " is " + x', hint: 'Declare variables with types.' },
      { title: 'Lesson 3: Functions and Recursion', code: 'func factorial(n: int) -> int [\n    if n <= 1 [ return 1 ]\n    return n * factorial(n - 1)\n]\nwrite factorial(5)', hint: 'Define a recursive function.' },
    ];
    for (const lesson of lessons) {
      console.log(chalk.bold(lesson.title));
      console.log(lesson.code);
      console.log(chalk.italic(lesson.hint));
      while (true) {
        const { userCode } = await inquirer.prompt([{ name: 'userCode', message: 'Your code (or \'skip\')' }]);
        if (userCode.toLowerCase() === 'skip') break;
        try {
          const output = runSubprocess([path.join(VIRA_BIN, 'vira-compiler'), 'eval', userCode], true);
          console.log(chalk.green(`Output: ${output}`));
          break;
        } catch {
          console.log(chalk.red('Error in code. Try again.'));
        }
      }
    }
    console.log(chalk.green('Tutorial complete! You\'re ready to code in Vira.'));
  });

program.command('version')
  .description('Show Vira version')
  .action(() => {
    const config = loadViraConfig();
    console.log(chalk.green(`Vira CLI version: ${config.version || '0.1.0'}`));
  });

program.command('version-bytes')
  .description('Show bytes.io repository version')
  .action(() => {
    const output = runSubprocess([path.join(VIRA_BIN, 'vira-packages'), 'version-bytes'], true);
    console.log(chalk.green(`bytes.io version: ${output || 'Unknown'}`));
  });

program.command('doctor')
  .description('Check environment and configuration')
  .action(() => {
    const table = new Table({ head: ['Check', 'Status', 'Details'], style: { head: ['magenta'] } });
    const checks = [
      ['VIRA_HOME', fs.existsSync(VIRA_HOME), VIRA_HOME],
      ['VIRA_BIN', fs.existsSync(VIRA_BIN), VIRA_BIN],
      ['vira-compiler', fs.existsSync(path.join(VIRA_BIN, 'vira-compiler')), 'Compiler binary'],
      ['vira-packages', fs.existsSync(path.join(VIRA_BIN, 'vira-packages')), 'Packages binary'],
      ['vira-parser_lexer', fs.existsSync(path.join(VIRA_BIN, 'vira-parser_lexer')), 'Parser/Lexer binary'],
      ['Node version', process.version.startsWith('v'), process.version],
      ['YAML config', fs.existsSync(VIRA_CONFIG), 'Global config'],
    ];
    let allPassed = true;
    checks.forEach(([check, status, details]) => {
      const statusText = status ? chalk.green('OK') : chalk.red('FAIL');
      table.push([chalk.cyan(check), statusText, chalk.yellow(details)]);
      if (!status) allPassed = false;
    });
    console.log(table.toString());
    console.log(allPassed ? chalk.green('System is healthy.') : chalk.red('Issues detected. Please resolve FAIL items.'));
  });

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

program.parse(process.argv);
