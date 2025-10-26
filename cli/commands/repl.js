const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('repl')
    .description('Start Vira REPL')
    .action(() => {
        console.log(chalk.yellow.bold('REPL placeholder: Starting Vira REPL...'));
        // utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-compiler'), 'repl']);
    });
}

module.exports = { addCommand };
