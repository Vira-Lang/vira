const path = require('path');
const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('fmt')
    .description('Format code')
    .action(() => {
        const config = utils.loadBytesYml();
        const sourceDir = path.join(process.cwd(), config['<>'] || 'cmd');
        const files = utils.getFiles(sourceDir, '.vira');
        const spinner = utils.startSpinner('Formatting files...');
        files.forEach(file => {
            utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-parser_lexer'), 'fmt', file]);
        });
        spinner.succeed(chalk.green.bold('Formatting complete!'));
    });
}

module.exports = { addCommand };
