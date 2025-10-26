const path = require('path');
const chalk = require('chalk');
const inquirer = require('inquirer');

function addCommand(program, utils) {
    program.command('tutorial')
    .description('Interactive tutorial')
    .action(async () => {
        console.log(chalk.rainbow.bold('Welcome to Vira Interactive Tutorial!')); // Using chalk for rainbow effect if supported, else fallback
        const lessons = [
            { title: 'Lesson 1: Hello World', code: 'func main() [ write "Hello, Vira!" ]', hint: 'Write a simple hello world.' },
            { title: 'Lesson 2: Variables and Types', code: 'let x: int = 42\nlet y: string = "Answer"\nwrite y + " is " + x', hint: 'Declare variables with types.' },
            { title: 'Lesson 3: Functions and Recursion', code: 'func factorial(n: int) -> int [\n    if n <= 1 [ return 1 ]\n    return n * factorial(n - 1)\n]\nwrite factorial(5)', hint: 'Define a recursive function.' },
        ];
        for (const lesson of lessons) {
            console.log(chalk.bold.magenta(lesson.title));
            console.log(chalk.white(lesson.code));
            console.log(chalk.italic.cyan(lesson.hint));
            while (true) {
                const { userCode } = await inquirer.prompt([{ name: 'userCode', message: chalk.cyan('Your code (or \'skip\')') }]);
                if (userCode.toLowerCase() === 'skip') break;
                try {
                    const output = utils.runSubprocess([path.join(utils.constants().VIRA_BIN, 'vira-compiler'), 'eval', userCode], true);
                    console.log(chalk.green.bold(`Output: ${chalk.white(output)}`));
                    break;
                } catch {
                    console.log(chalk.red.bold('Error in code. Try again.'));
                }
            }
        }
        console.log(chalk.green.bold('Tutorial complete! You\'re ready to code in Vira.'));
    });
}

module.exports = { addCommand };
