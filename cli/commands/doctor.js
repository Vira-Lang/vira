const path = require('path');
const fs = require('fs');
const chalk = require('chalk');
const Table = require('cli-table3');

function addCommand(program, utils) {
    program.command('doctor')
    .description('Check environment and configuration')
    .action(() => {
        const table = new Table({
            head: [chalk.magenta('Check'), chalk.magenta('Status'), chalk.magenta('Details')],
                                style: { head: ['magenta'], border: ['grey'] }
        });
        const consts = utils.constants();
        const checks = [
            ['VIRA_HOME', fs.existsSync(consts.VIRA_HOME), chalk.white(consts.VIRA_HOME)],
            ['VIRA_BIN', fs.existsSync(consts.VIRA_BIN), chalk.white(consts.VIRA_BIN)],
            ['vira-compiler', fs.existsSync(path.join(consts.VIRA_BIN, 'vira-compiler')), chalk.cyan('Compiler binary')],
            ['vira-packages', fs.existsSync(path.join(consts.VIRA_BIN, 'vira-packages')), chalk.cyan('Packages binary')],
            ['vira-parser_lexer', fs.existsSync(path.join(consts.VIRA_BIN, 'vira-parser_lexer')), chalk.cyan('Parser/Lexer binary')],
            ['Node version', process.version.startsWith('v18'), chalk.white(process.version)],
            ['YAML config', fs.existsSync(consts.VIRA_CONFIG), chalk.cyan('Global config')],
        ];
        let allPassed = true;
        checks.forEach(([check, status, details]) => {
            const statusText = status ? chalk.green.bold('OK') : chalk.red.bold('FAIL');
            table.push([chalk.cyan(check), statusText, details]);
            if (!status) allPassed = false;
        });
            console.log(table.toString());
            console.log(allPassed ? chalk.green.bold('System is healthy!') : chalk.red.bold('Issues detected. Please resolve FAIL items.'));
    });
}

module.exports = { addCommand };
