var io = require('io'),
    fs = require('fs');

console.log('Hello, world')

var test = require('./test');

var file = new fs.File("./test_file.txt", 'wr+');

file.write("Hello, Dean!: " + test.hello);

io.stdout.write("Hello, World from js\n").flush();

var buffer = io.stdin.readLine();
file.write(buffer).close();