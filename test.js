
// process.stdout.write("Hello, World\n");
// process.stdout.write(JSON.stringify(require('./test.json')) + "\n");

var test = new Test();

process.stdout.write((test instanceof Test).toString());
process.stdout.write((test instanceof Parent).toString());


module.exports = {
    test: 'Hello, World',
    json: require('./tests/cheese'),
    args: process.args,
    cwd: process.cwd,
}