var fs = require('fs');

var file = new fs.File('duk_history.txt');

var buffer = file.readAll();

require('io').stdout.write(new TextDecoder('utf8').decode(buffer));