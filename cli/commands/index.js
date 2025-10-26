const fs = require('fs');
const path = require('path');

function registerCommands(program, utils) {
    const commandsDir = path.join(__dirname);
    fs.readdirSync(commandsDir).forEach(file => {
        if (file !== 'index.js' && file.endsWith('.js')) {
            const commandModule = require(path.join(commandsDir, file));
            if (typeof commandModule.addCommand === 'function') {
                commandModule.addCommand(program, utils);
            }
        }
    });
}

module.exports = registerCommands;
