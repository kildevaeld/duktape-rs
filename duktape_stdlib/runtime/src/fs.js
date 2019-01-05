exports.readFile = function (path) {
    const file = new exports.File(path, 'r');
    const data = file.readText();
    file.close();
    return data;
}