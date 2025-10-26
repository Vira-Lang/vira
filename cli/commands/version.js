const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('version')
    .description('Show Vira version')
    .action(() => {
        const config = utils.loadViraConfig();
        console.log(chalk.green.bold(`Vira CLI version: ${chalk.white(config.version || '0.1.0')}`));
    });
}

module.exports = { addCommand };
