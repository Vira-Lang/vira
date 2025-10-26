const path = require('path');
const fs = require('fs');
const commander = require('commander');
const chalk = require('chalk');

const utils = require('./utils/utils');
const registerCommands = require('./commands/index');

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

// Export constants for use in other modules
utils.setConstants({ VIRA_HOME, VIRA_BIN, VIRA_LIBS, VIRA_LOGS, VIRA_CACHE, VIRA_CONFIG });

const program = new commander.Command('vira')
.description('Vira CLI - A colorful command-line interface for Vira language')
.version('0.1.0')
.option('-v, --verbose', 'Enable verbose mode', false);

registerCommands(program, utils);

program.parse(process.argv);

if (!process.argv.slice(2).length) {
  program.outputHelp(chalk.blue);
}
