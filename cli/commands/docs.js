const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('docs')
    .description('Show documentation')
    .action(() => {
        const docs = chalk.blue.bold(`Vira Documentation\n\n`) +
        chalk.cyan('- Syntax: Use [ ] for blocks\n') +
        chalk.cyan('- Types: Static by default\n') +
        chalk.white('For full docs, visit bytes.io');
        console.log(chalk.green(docs));
    });
}

module.exports = { addCommand };
