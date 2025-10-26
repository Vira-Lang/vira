const path = require('path');
const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('refresh')
    .description('Refresh repository cache')
    .action(() => {
        const spinner = utils.startSpinner('Refreshing repository...');
        utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-packages'), 'refresh']);
        spinner.succeed(chalk.green.bold('Repository refreshed!'));
    });
}

module.exports = { addCommand };
