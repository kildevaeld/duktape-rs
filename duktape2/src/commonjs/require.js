(function (_require) {

    const protocolReg = /^([a-zA-Z0-9]+)(?::\/\/)(\/?[a-zA-Z0-9\.\-]+(?:\/[a-zA-Z0-9\.\-]+)*)$/i,
        fileReg = /^(?:\/|\.\.?\/)(?:[^\/\\0]+(?:\/)?)*$/i;




    function require(name) {
        if (_require.cache[name]) {
            return _require.cache[name];
        }

        if (fileReg.test(name)) {
            name = "file://" + name;
        }

        var match = name.match(protocolReg);
        //if (!match) throw new TypeError("invalid protocol");

        return _require({
            protocol: match ? match[1] : null,
            id: match ? match[2] : name
        });

    }

    return require;
});