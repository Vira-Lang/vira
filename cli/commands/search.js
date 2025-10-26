const path = require('path');
const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('search')
    .description('Search for libraries in repo')
    .argument('<query...>', 'Search query')
    .action((query) => {
        const output = utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-packages'), 'search', query.join(' ')], true);
        console.log(chalk.blue.bold('Search Results:\n') + chalk.white(output));
    });
}

module.exports = { addCommand };
