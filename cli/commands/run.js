const path = require('path');
const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('run')
    .description('Run Vira code in VM')
    .argument('<file>', 'File or directory to run')
    .action((file) => {
        const spinner = utils.startSpinner('Running your code...');
        utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-compiler'), 'run', file], false, 300000);
        spinner.succeed(chalk.green.bold('Run complete!'));
    });
}

module.exports = { addCommand };
