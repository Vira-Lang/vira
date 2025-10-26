const path = require('path');
const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('version-bytes')
    .description('Show bytes.io repository version')
    .action(() => {
        const output = utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-packages'), 'version-bytes'], true);
        console.log(chalk.green.bold(`bytes.io version: ${chalk.white(output || 'Unknown')}`));
    });
}

module.exports = { addCommand };
