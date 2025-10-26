const path = require('path');
const chalk = require('chalk');
const inquirer = require('inquirer');

function addCommand(program, utils) {
    program.command('install')
    .description('Install packages')
    .argument('[packages...]', 'Packages to install')
    .option('--in-project', 'Install in project')
    .action(async (packages, options) => {
        if (!packages.length) {
            const config = utils.loadBytesYml();
            const deps = config.dependencies || {};
            packages = Object.entries(deps).map(([dep, ver]) => `${dep}@${ver}`);
            if (!packages.length) {
                console.log(chalk.red.bold('No packages specified and no dependencies in bytes.yml.'));
                process.exit(1);
            }
        }
        for (const pkg of packages) {
            const spinner = utils.startSpinner(`Installing ${chalk.white(pkg)}...`);
            const cmd = [path.join(utils.constants().VIRA_BIN, 'vira-packages'), 'install', pkg];
            if (options.inProject) cmd.push('--in-project');
            utils.runSubprocess(cmd);
            spinner.succeed(chalk.green.bold(`Installed ${chalk.white(pkg)}`));
        }
        console.log(chalk.green.bold('Installation complete!'));
    });
}

module.exports = { addCommand };
