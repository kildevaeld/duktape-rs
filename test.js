// var fs = require('fs');

// var file = new fs.File('duk_history.txt');

// var buffer = file.readAll();

// require('io').stdout.write(new TextDecoder('utf8').decode(buffer));
process.stdout.write("Hello, World\n");
process.stdout.write(JSON.stringify(require('./test.json')) + "\n");
module.exports = {
    test: 'Hello, World',
    json: require('./tests/cheese'),
    args: process.args,
    cwd: process.cwd,
}