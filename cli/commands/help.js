const chalk = require('chalk');
const Table = require('cli-table3');

function addCommand(program, utils) {
    program.command('help')
    .description('Display list of commands')
    .action(() => {
        const table = new Table({
            head: [chalk.magenta('Command'), chalk.magenta('Description')],
                                style: { head: ['magenta'], border: ['grey'] }
        });
        program.commands.forEach(cmd => {
            table.push([chalk.cyan.bold(cmd.name()), chalk.green(cmd.description())]);
        });
        console.log(table.toString());
    });
}

module.exports = { addCommand };
