module.exports.Reader.prototype.readText = function (decoder) {
    decoder = decoder || new TextDecoder('utf-8');
    const buf = this.readAll();
    return decoder.decode(buf);
};

module.exports.Reader.prototype.readJSON = function (decoder) {
    return JSON.parse(this.readText(decoder));
};

Object.freeze(module.exports);