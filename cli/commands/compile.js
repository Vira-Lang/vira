const path = require('path');
const fs = require('fs');
const chalk = require('chalk');

function addCommand(program, utils) {
    program.command('compile')
    .description('Compile Vira code')
    .option('--platform <plat>', 'Target platform', utils.getPlatform())
    .option('--output <out>', 'Output directory', 'build')
    .action((cmd) => {
        const spinner = utils.startSpinner('Compiling your code...');
        const config = utils.loadBytesYml();
        utils.resolveDependencies(config);
        const sourceDir = path.join(process.cwd(), config['<>'] || 'cmd');
        const outputDir = path.join(process.cwd(), cmd.output);
        fs.mkdirSync(outputDir, { recursive: true });
        utils.runSubprocess([
            path.join(utils.constants().VIRA_BIN, 'vira-compiler'),
                            'compile', sourceDir, '--platform', cmd.platform, '--output', outputDir
        ]);
        spinner.succeed(chalk.green.bold(`Compilation complete! Output in ${chalk.white(cmd.output)}/`));
    });
}

module.exports = { addCommand };
