const path = require('path');
const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('remove')
    .description('Remove packages')
    .argument('<packages...>', 'Packages to remove')
    .action((packages) => {
        for (const pkg of packages) {
            const spinner = utils.startSpinner(`Removing ${chalk.white(pkg)}...`);
            utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-packages'), 'remove', pkg]);
            spinner.succeed(chalk.green.bold(`Removed ${chalk.white(pkg)}`));
        }
        console.log(chalk.green.bold('Removal complete!'));
    });
}

module.exports = { addCommand };
