const path = require('path');
const fs = require('fs');
const chalk = require('chalk');
const inquirer = require('inquirer');
const yaml = require('js-yaml');

function addCommand(program, utils) {
    program.command('init')
    .description('Initialize a new Vira project')
    .action(async () => {
        const existing = utils.findBytesYml();
        if (existing) {
            const { reinit } = await inquirer.prompt([{
                type: 'confirm',
                name: 'reinit',
                message: chalk.yellow('Project already initialized. Reinitialize?'),
                                                     default: false
            }]);
            if (!reinit) return;
        }
        const answers = await inquirer.prompt([
            { name: 'name', message: chalk.cyan('Project name'), default: path.basename(process.cwd()) },
                                              { name: 'author', message: chalk.cyan('Author'), default: process.env.USER || 'unknown' },
                                              { name: 'description', message: chalk.cyan('Description'), default: '' },
        ]);
        const bytesYml = {
            name: answers.name,
            version: '0.1.0',
            author: answers.author,
            description: answers.description,
            '<>': 'cmd',
            dependencies: {},
            'dev-dependencies': {}
        };
        fs.writeFileSync('bytes.yml', yaml.dump(bytesYml));
        fs.mkdirSync('cmd', { recursive: true });
        fs.writeFileSync(path.join('cmd', 'main.vira'), `<io>

        @ Hello Vira program
        func main()
        [
            let msg: string = "Hello, Vira!"
            write msg
        ]
        `);
        fs.mkdirSync('tests', { recursive: true });
        console.log(chalk.green.bold('Project initialized successfully with colorful setup!'));
    });
}

module.exports = { addCommand };
