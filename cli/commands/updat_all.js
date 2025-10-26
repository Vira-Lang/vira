const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('updat?')
    .description('Update Vira binaries and libraries')
    .action(() => {
        const updateCmd = program.commands.find(c => c.name() === 'update');
        const upgradeCmd = program.commands.find(c => c.name() === 'upgrade');
        if (updateCmd) updateCmd.action()();
        if (upgradeCmd) upgradeCmd.action()();
        console.log(chalk.green.bold('Full update complete!'));
    });
}

module.exports = { addCommand };
