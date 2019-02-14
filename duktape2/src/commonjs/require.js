(function (_require) {

    const protocolReg = /c/i

    function require(name) {
        if (_require.cache[name]) {
            return _require.cache[name];
        }
    }

    return require;
});