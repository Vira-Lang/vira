const path = require('path');
const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('update')
    .description('Update packages')
    .action(() => {
        const spinner = utils.startSpinner('Updating packages...');
        utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-packages'), 'update']);
        spinner.succeed(chalk.green.bold('Packages updated!'));
    });
}

module.exports = { addCommand };
