const path = require('path');
const fs = require('fs');
const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('check')
    .description('Check .vira code and bytes.yml')
    .action(() => {
        const config = utils.loadBytesYml();
        const requiredKeys = ['name', 'version'];
        const missing = requiredKeys.filter(k => !(k in config));
        if (missing.length) {
            console.log(chalk.red.bold(`Invalid bytes.yml: missing ${chalk.white(missing.join(', '))}.`));
            process.exit(1);
        }
        const sourceDir = path.join(process.cwd(), config['<>'] || 'cmd');
        if (!fs.existsSync(sourceDir)) {
            console.log(chalk.red.bold('Source directory not found.'));
            process.exit(1);
        }
        const files = utils.getFiles(sourceDir, '.vira');
        const spinner = utils.startSpinner('Checking files...');
        files.forEach(file => {
            utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-parser_lexer'), 'check', file]);
        });
        spinner.succeed(chalk.green.bold('All checks passed!'));
    });
}

module.exports = { addCommand };
