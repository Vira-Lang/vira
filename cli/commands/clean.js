const path = require('path');
const fs = require('fs');
const chalk = require('chalk');
const inquirer = require('inquirer');
const rimraf = require('rimraf');

function addCommand(program, utils) {
    program.command('clean')
    .description('Clean build artifacts')
    .option('--global', 'Clean global cache')
    .action(async (options) => {
        const buildDir = path.join(process.cwd(), 'build');
        if (fs.existsSync(buildDir)) {
            const { confirm } = await inquirer.prompt([{
                type: 'confirm',
                name: 'confirm',
                message: chalk.yellow(`Clean ${chalk.white(buildDir)}?`),
                                                      default: true
            }]);
            if (confirm) {
                rimraf.sync(buildDir);
                console.log(chalk.green.bold(`Cleaned ${chalk.white(buildDir)}`));
            }
        }
        if (options.global) {
            const { confirmGlobal } = await inquirer.prompt([{
                type: 'confirm',
                name: 'confirmGlobal',
                message: chalk.yellow('Clean global cache and logs?'),
                                                      default: false
            }]);
            if (confirmGlobal) {
                rimraf.sync(utils.constants().VIRA_CACHE);
                fs.mkdirSync(utils.constants().VIRA_CACHE);
                console.log(chalk.green.bold('Cleaned global cache'));
                fs.readdirSync(utils.constants().VIRA_LOGS).forEach(file => fs.unlinkSync(path.join(utils.constants().VIRA_LOGS, file)));
                console.log(chalk.green.bold('Cleaned logs'));
            }
        }
        console.log(chalk.green.bold('Clean complete!'));
    });
}

module.exports = { addCommand };
