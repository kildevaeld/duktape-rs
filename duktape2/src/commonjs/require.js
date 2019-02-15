(function (_require) {
    const protocolReg = /^([a-zA-Z0-9]+)(?::\/\/)(\/?[a-zA-Z0-9\.\-]+(?:\/[a-zA-Z0-9\.\-]+)*)$/i,
        fileReg = /^(?:\/|\.\.?\/)(?:[^\/\\0]+(?:\/)?)*$/i;

    function require(name) {
        if (fileReg.test(name)) {
            name = "file://" + name;
        }
        const match = name.match(protocolReg);
        return _require({
            protocol: match ? match[1] : null,
            id: match ? match[2] : name
        });
    }
    return require;
});