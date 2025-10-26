const path = require('path');
const fs = require('fs');
const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('test')
    .description('Run tests')
    .action(() => {
        const config = utils.loadBytesYml();
        const testDir = path.join(process.cwd(), config.test_dir || 'tests');
        if (!fs.existsSync(testDir)) {
            console.log(chalk.red.bold('Tests directory not found.'));
            process.exit(1);
        }
        const spinner = utils.startSpinner('Running tests...');
        utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-compiler'), 'test', testDir]);
        spinner.succeed(chalk.green.bold('Tests complete!'));
    });
}

module.exports = { addCommand };
