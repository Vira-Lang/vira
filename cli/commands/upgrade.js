const path = require('path');
const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('upgrade')
    .description('Upgrade Vira language binaries')
    .action(() => {
        const spinner = utils.startSpinner('Upgrading Vira...');
        utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-packages'), 'upgrade']);
        spinner.succeed(chalk.green.bold('Vira upgraded!'));
    });
}

module.exports = { addCommand };
